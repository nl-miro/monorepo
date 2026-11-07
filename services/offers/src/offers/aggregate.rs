use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use retailer_sauce::availability::Availability;
use retailer_sauce::item_condition::ItemCondition;
use testimony::prelude::{Aggregate, EventEnvelope};

use super::event::{
    OfferAvailabilityChangedEvent, OfferDescriptionChangedEvent, OfferGtinChangedEvent,
    OfferHistoricPriceAddedEvent, OfferImageAddedEvent, OfferImageRemovedEvent,
    OfferItemConditionChangedEvent, OfferLanguageChangedEvent, OfferPriceChangedEvent,
    OfferPriceConfirmedEvent, OfferShopifyProductGidChangedEvent,
    OfferShopifyProductVariantGidChangedEvent, OfferSkuChangedEvent, OfferTargetMarketChangedEvent,
    OfferTitleChangedEvent,
};
use super::event::{OfferCreatedEvent, RetailerOfferEvent};
use super::history_error::OfferHistoryError;
use super::retailer_offer_id::RetailerOfferId;
use super::state::{Content, ExternalOfferIdentifiers, Price, Pricing, RetailerOfferState};
use super::target_market::TargetMarket;

pub struct RetailerOfferAggregate {
    pub state: RetailerOfferState,
    pub sequence: u64,
    pub uncommitted_events: Vec<EventEnvelope<RetailerOfferEvent>>,
}

impl Aggregate for RetailerOfferAggregate {
    type Event = RetailerOfferEvent;
    type Id = RetailerOfferId;
    type HistoryError = OfferHistoryError;

    fn from_history(
        id: Self::Id,
        events: Vec<EventEnvelope<Self::Event>>,
    ) -> Result<Self, Self::HistoryError> {
        let mut events = events.into_iter();
        let first = events.next().ok_or_else(OfferHistoryError::empty)?;
        OfferHistoryError::guard_envelope(&id, 1, &first)?;
        let RetailerOfferEvent::OfferCreated(created) = first.event() else {
            return Err(OfferHistoryError::first(
                "OfferCreatedNextGeneration",
                first.event(),
            ));
        };
        let mut aggregate = Self {
            state: Self::when_created(created),
            sequence: 1,
            uncommitted_events: Vec::new(),
        };
        for envelope in events {
            OfferHistoryError::guard_envelope(
                &id,
                OfferHistoryError::next(aggregate.sequence)?,
                &envelope,
            )?;
            aggregate.guard_replay(envelope.event(), envelope.sequence())?;
            aggregate.when(envelope.event());
            aggregate.sequence = envelope.sequence();
        }
        Ok(aggregate)
    }

    fn uncommitted_events(&self) -> &[EventEnvelope<Self::Event>] {
        &self.uncommitted_events
    }

    fn mark_events_committed(&mut self) {
        self.uncommitted_events.clear();
    }
}

impl RetailerOfferAggregate {
    //
    /// Create an offer with its first price observation and no earlier price history.
    //
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: RetailerOfferId,
        retailer_code: RetailerCode,
        url: String,
        identifiers: ExternalOfferIdentifiers,
        target_market: TargetMarket,
        language: String,
        content: Content,
        price: Price,
        availability: Option<Availability>,
        item_condition: Option<ItemCondition>,
        created_at: DateTime<Utc>,
    ) -> Self {
        let event = OfferCreatedEvent::new(
            id,
            retailer_code,
            url,
            identifiers,
            target_market,
            language,
            content,
            price,
            availability,
            item_condition,
            created_at,
        );

        // The aggregate always holds initialized state, including before its first apply.
        let mut aggregate = Self {
            state: Self::when_created(&event),
            sequence: 0,
            uncommitted_events: Vec::new(),
        };
        aggregate.apply(RetailerOfferEvent::OfferCreated(event));
        aggregate
    }
}

impl RetailerOfferAggregate {
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        identifiers: ExternalOfferIdentifiers,
        target_market: TargetMarket,
        language: String,
        content: Content,
        price: Price,
        availability: Option<Availability>,
        item_condition: Option<ItemCondition>,
        updated_at: DateTime<Utc>,
    ) {
        let mut events = Vec::new();

        //
        // GTIN changes
        //
        let gtin_event = self.check_for_gtin_changes(&identifiers.gtin, updated_at);
        events.extend(gtin_event);

        //
        // SKU changes
        //
        let sku_event = self.check_for_sku_changes(&identifiers.sku, updated_at);
        events.extend(sku_event);

        //
        // Shopify product GID changes
        //
        let shopify_product_gid_event = self
            .check_for_shopify_product_gid_changes(&identifiers.shopify_product_gid, updated_at);
        events.extend(shopify_product_gid_event);

        //
        // Shopify product variant GID changes
        //
        let shopify_product_variant_gid_event = self.check_for_shopify_product_variant_gid_changes(
            &identifiers.shopify_product_variant_gid,
            updated_at,
        );
        events.extend(shopify_product_variant_gid_event);

        //
        // Target market changes
        //
        let target_market_event = self.check_for_target_market_changes(target_market, updated_at);
        events.extend(target_market_event);

        //
        // Language changes
        //
        let language_event = self.check_for_language_changes(&language, updated_at);
        events.extend(language_event);

        //
        // Title changes
        //
        let title_event = self.check_for_title_changes(&content.title);
        events.extend(title_event);

        //
        // Description changes
        //
        let description_event =
            self.check_for_description_changes(&content.description, updated_at);
        events.extend(description_event);

        //
        // Image changes
        //
        let (_, image_events) = self.check_for_image_changes(&content.images, updated_at);
        events.extend(image_events);

        //
        // Price changes
        //
        let price_event = self.check_for_price_changes(&price, updated_at);
        events.push(price_event);

        //
        // Availability changes
        //
        let availability_event = self.check_for_availability_changes(&availability, updated_at);
        events.extend(availability_event);

        //
        // Item condition changes
        //
        let item_condition_event =
            self.check_for_item_condition_changes(&item_condition, updated_at);
        events.extend(item_condition_event);

        // Check the whole batch before applying any events.
        self.sequence
            .checked_add(events.len() as u64)
            .expect("offer sequence exhausted");

        for event in events {
            self.apply(event);
        }
    }
}

impl RetailerOfferAggregate {
    /// Apply and record a newly raised domain event.
    fn apply(&mut self, event: RetailerOfferEvent) {
        let sequence = self
            .sequence
            .checked_add(1)
            .expect("offer sequence exhausted");
        self.when(&event);
        self.sequence = sequence;
        self.uncommitted_events
            .push(EventEnvelope::new(event, sequence));
    }

    fn guard_replay(
        &self,
        event: &RetailerOfferEvent,
        sequence: u64,
    ) -> Result<(), OfferHistoryError> {
        match event {
            RetailerOfferEvent::OfferCreated(_) => {
                return Err(OfferHistoryError::duplicate(sequence));
            }
            RetailerOfferEvent::OfferImageAdded(image) => {
                let position = usize::try_from(image.position)
                    .map_err(|error| OfferHistoryError::invalid(sequence, error))?;
                if position > self.state.content.images.len() {
                    return Err(OfferHistoryError::invalid(
                        sequence,
                        format!(
                            "image position {} exceeds image count {}",
                            image.position,
                            self.state.content.images.len()
                        ),
                    ));
                }
            }
            RetailerOfferEvent::OfferGtinChanged(_)
            | RetailerOfferEvent::OfferSkuChanged(_)
            | RetailerOfferEvent::OfferShopifyProductGidChanged(_)
            | RetailerOfferEvent::OfferShopifyProductVariantGidChanged(_)
            | RetailerOfferEvent::OfferTargetMarketChanged(_)
            | RetailerOfferEvent::OfferLanguageChanged(_)
            | RetailerOfferEvent::OfferDescriptionChanged(_)
            | RetailerOfferEvent::OfferAvailabilityChanged(_)
            | RetailerOfferEvent::OfferItemConditionChanged(_)
            | RetailerOfferEvent::OfferTitleChanged(_)
            | RetailerOfferEvent::OfferPriceChanged(_)
            | RetailerOfferEvent::OfferPriceConfirmed(_)
            | RetailerOfferEvent::OfferHistoricPriceAdded(_)
            | RetailerOfferEvent::OfferImageRemoved(_) => {}
        }
        Ok(())
    }

    /// Evolve state without recording events or changing the stream sequence.
    fn when(&mut self, event: &RetailerOfferEvent) {
        match event {
            RetailerOfferEvent::OfferGtinChanged(event) => self.when_gtin_changed(event),
            RetailerOfferEvent::OfferSkuChanged(event) => self.when_sku_changed(event),
            RetailerOfferEvent::OfferShopifyProductGidChanged(event) => {
                self.when_shopify_product_gid_changed(event)
            }
            RetailerOfferEvent::OfferShopifyProductVariantGidChanged(event) => {
                self.when_shopify_product_variant_gid_changed(event)
            }
            RetailerOfferEvent::OfferTargetMarketChanged(event) => {
                self.when_target_market_changed(event)
            }
            RetailerOfferEvent::OfferLanguageChanged(event) => self.when_language_changed(event),
            RetailerOfferEvent::OfferDescriptionChanged(event) => {
                self.when_description_changed(event)
            }
            RetailerOfferEvent::OfferAvailabilityChanged(event) => {
                self.when_availability_changed(event)
            }
            RetailerOfferEvent::OfferItemConditionChanged(event) => {
                self.when_item_condition_changed(event)
            }
            RetailerOfferEvent::OfferTitleChanged(event) => self.when_title_changed(event),
            RetailerOfferEvent::OfferPriceChanged(event) => self.when_price_changed(event),
            RetailerOfferEvent::OfferPriceConfirmed(event) => self.when_price_confirmed(event),
            RetailerOfferEvent::OfferHistoricPriceAdded(event) => {
                self.when_historic_price_added(event)
            }
            RetailerOfferEvent::OfferImageAdded(event) => self.when_image_added(event),
            RetailerOfferEvent::OfferImageRemoved(event) => self.when_image_removed(event),
            RetailerOfferEvent::OfferCreated(event) => self.state = Self::when_created(event),
        }
    }

    fn when_gtin_changed(&mut self, event: &OfferGtinChangedEvent) {
        self.state.identifiers.gtin = event.gtin.clone();
        self.state.updated_at = event.updated_at;
    }

    fn when_sku_changed(&mut self, event: &OfferSkuChangedEvent) {
        self.state.identifiers.sku = event.sku.clone();
        self.state.updated_at = event.updated_at;
    }

    fn when_shopify_product_gid_changed(&mut self, event: &OfferShopifyProductGidChangedEvent) {
        self.state.identifiers.shopify_product_gid = event.shopify_product_gid.clone();
        self.state.updated_at = event.updated_at;
    }

    fn when_shopify_product_variant_gid_changed(
        &mut self,
        event: &OfferShopifyProductVariantGidChangedEvent,
    ) {
        self.state.identifiers.shopify_product_variant_gid =
            event.shopify_product_variant_gid.clone();
        self.state.updated_at = event.updated_at;
    }

    fn when_target_market_changed(&mut self, event: &OfferTargetMarketChangedEvent) {
        self.state.target_market = event.target_market;
        self.state.updated_at = event.updated_at;
    }

    fn when_language_changed(&mut self, event: &OfferLanguageChangedEvent) {
        self.state.language = event.language.clone();
        self.state.updated_at = event.updated_at;
    }

    fn when_description_changed(&mut self, event: &OfferDescriptionChangedEvent) {
        self.state.content.description = event.description.clone();
        self.state.updated_at = event.updated_at;
    }

    fn when_availability_changed(&mut self, event: &OfferAvailabilityChangedEvent) {
        self.state.availability = event.availability.clone();
        self.state.updated_at = event.updated_at;
    }

    fn when_item_condition_changed(&mut self, event: &OfferItemConditionChangedEvent) {
        self.state.item_condition = event.item_condition.clone();
        self.state.updated_at = event.updated_at;
    }

    fn when_title_changed(&mut self, event: &OfferTitleChangedEvent) {
        self.state.content.title = event.title.clone();
    }

    fn when_price_changed(&mut self, event: &OfferPriceChangedEvent) {
        let previous = std::mem::replace(&mut self.state.price.last, event.price.clone());
        self.state.price.history.push(previous);
        self.state.updated_at = event.updated_at;
    }

    fn when_price_confirmed(&mut self, event: &OfferPriceConfirmedEvent) {
        let previous = std::mem::replace(&mut self.state.price.last, event.price.clone());
        self.state.price.history.push(previous);
        self.state.updated_at = event.updated_at;
    }

    fn when_historic_price_added(&mut self, event: &OfferHistoricPriceAddedEvent) {
        self.state.price.history.push(event.historic_price.clone());
        self.state.updated_at = event.updated_at;
    }

    fn when_image_added(&mut self, event: &OfferImageAddedEvent) {
        self.state
            .content
            .images
            .insert(event.position as usize, event.image_url.clone());
    }

    fn when_image_removed(&mut self, event: &OfferImageRemovedEvent) {
        self.state
            .content
            .images
            .retain(|image| image != &event.image_url);
        self.state.updated_at = event.updated_at;
    }

    fn when_created(event: &OfferCreatedEvent) -> RetailerOfferState {
        RetailerOfferState::new(
            event.id.clone(),
            event.retailer_code,
            event.url.clone(),
            event.identifiers.clone(),
            event.target_market,
            event.language.clone(),
            event.content.clone(),
            Pricing::new(event.price.clone()),
            event.availability.clone(),
            event.item_condition.clone(),
            event.created_at,
        )
    }
}

impl RetailerOfferAggregate {
    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_gtin_changes(&self,gtin: &Option<String>,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.identifiers.gtin != *gtin).then(|| ())?;

        let evt = OfferGtinChangedEvent::new(
            self.state.id.clone(),
            self.state.identifiers.gtin.clone(),
            gtin.clone(),
            updated_at,
        );

        Some(RetailerOfferEvent::OfferGtinChanged(evt))
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_sku_changes(&self,sku: &Option<String>,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.identifiers.sku != *sku).then(|| ())?;

        let evt = OfferSkuChangedEvent::new(
            self.state.id.clone(),
            self.state.identifiers.sku.clone(),
            sku.clone(),
            updated_at,
        );

        Some(RetailerOfferEvent::OfferSkuChanged(evt))
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_shopify_product_gid_changes(&self,shopify_product_gid: &Option<String>,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.identifiers.shopify_product_gid != *shopify_product_gid).then(|| ())?;

        let evt = OfferShopifyProductGidChangedEvent::new(
            self.state.id.clone(),
            self.state.identifiers.shopify_product_gid.clone(),
            shopify_product_gid.clone(),
            updated_at,
        );

        Some(RetailerOfferEvent::OfferShopifyProductGidChanged(evt))
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_shopify_product_variant_gid_changes(&self,shopify_product_variant_gid: &Option<String>,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.identifiers.shopify_product_variant_gid != *shopify_product_variant_gid).then(|| ())?;

        let evt = OfferShopifyProductVariantGidChangedEvent::new(
            self.state.id.clone(),
            self.state.identifiers.shopify_product_variant_gid.clone(),
            shopify_product_variant_gid.clone(),
            updated_at,
        );

        Some(RetailerOfferEvent::OfferShopifyProductVariantGidChanged(evt))
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_target_market_changes(&self,target_market: TargetMarket,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.target_market != target_market).then(|| ())?;

        let evt = OfferTargetMarketChangedEvent::new(
            self.state.id.clone(),
            self.state.target_market,
            target_market,
            updated_at,
        );

        Some(RetailerOfferEvent::OfferTargetMarketChanged(evt))
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_language_changes(&self,language: &str,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.language != language).then(|| ())?;

        let evt = OfferLanguageChangedEvent::new(
            self.state.id.clone(),
            self.state.language.clone(),
            language.to_owned(),
            updated_at,
        );

        Some(RetailerOfferEvent::OfferLanguageChanged(evt))
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_title_changes(&self, title: &str) -> Option<RetailerOfferEvent> {

        (self.state.content.title != title).then(|| ())?;

        let evt = OfferTitleChangedEvent::new(self.state.id.clone(), title.to_owned());

        Some(RetailerOfferEvent::OfferTitleChanged(evt))
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_description_changes(&self,description: &str,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.content.description != description).then(|| ())?;

        let evt = OfferDescriptionChangedEvent::new(
            self.state.id.clone(),
            self.state.content.description.clone(),
            description.to_owned(),
            updated_at,
        );

        Some(RetailerOfferEvent::OfferDescriptionChanged(evt))
    }


    #[rustfmt::skip]
    fn check_for_image_changes(&self, images: &[String], updated_at: DateTime<Utc>) -> (Vec<String>, Vec<RetailerOfferEvent>) {
        let mut next_images = self.state.content.images.clone();
        let mut events = Vec::new();
        for image in images {
            if next_images.contains(image) {
                continue;
            }
            let position =
                u32::try_from(next_images.len()).expect("offer image position exhausted");
            let evt = OfferImageAddedEvent::new(self.state.id.clone(), image.clone(), position);
            events.push(RetailerOfferEvent::OfferImageAdded(evt));
            next_images.push(image.clone());
        }
        for image in &self.state.content.images {
            if images.contains(image) || !next_images.contains(image) {
                continue;
            }
            let evt = OfferImageRemovedEvent::new(self.state.id.clone(), image.clone(), updated_at);
            events.push(RetailerOfferEvent::OfferImageRemoved(evt));
            next_images.retain(|current| current != image);
        }

        (next_images, events)
    }

    #[rustfmt::skip]
    fn check_for_price_changes(&self, price: &Price, updated_at: DateTime<Utc>) -> RetailerOfferEvent {
        let last = &self.state.price.last;

        // TODO: Remove?
        // Older observations add historic prices
        //
        if price.scraped_at < last.scraped_at {
            let evt =
                OfferHistoricPriceAddedEvent::new(self.state.id.clone(), price.clone(), updated_at);

            return RetailerOfferEvent::OfferHistoricPriceAdded(evt);
        }

        //
        // Current or previous price changes
        //
        if price.current != last.current || price.previous != last.previous {
            let evt = OfferPriceChangedEvent::new(
                self.state.id.clone(),
                last.clone(),
                price.clone(),
                updated_at,
            );

            return RetailerOfferEvent::OfferPriceChanged(evt);
        }

        //
        // Unchanged prices are confirmed
        //
        let evt = OfferPriceConfirmedEvent::new(self.state.id.clone(), price.clone(), updated_at);

        RetailerOfferEvent::OfferPriceConfirmed(evt)
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_availability_changes(&self,availability: &Option<Availability>,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.availability != *availability).then(|| ())?;

        let evt = OfferAvailabilityChangedEvent::new(
            self.state.id.clone(),
            self.state.availability.clone(),
            availability.clone(),
            updated_at,
        );

        Some(RetailerOfferEvent::OfferAvailabilityChanged(
            evt,
        ))
    }

    #[rustfmt::skip]
    #[expect(clippy::unnecessary_lazy_evaluations, reason = "Keep then-based change guards consistent")]
    fn check_for_item_condition_changes(&self,item_condition: &Option<ItemCondition>,updated_at: DateTime<Utc>) -> Option<RetailerOfferEvent> {

        (self.state.item_condition != *item_condition).then(|| ())?;

        let evt = OfferItemConditionChangedEvent::new(
            self.state.id.clone(),
            self.state.item_condition.clone(),
            item_condition.clone(),
            updated_at,
        );

        Some(RetailerOfferEvent::OfferItemConditionChanged(
            evt,
        ))
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod identifier_tests;

#[cfg(test)]
mod target_market_tests;

#[cfg(test)]
mod language_tests;

#[cfg(test)]
mod content_tests;

#[cfg(test)]
mod pricing_tests;

#[cfg(test)]
mod status_tests;

#[cfg(test)]
mod commit_tests;
