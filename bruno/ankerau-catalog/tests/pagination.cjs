const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { test } = require('node:test');

const root = path.resolve(__dirname, '..');
const collection = fs.readFileSync(path.join(root, 'collection.bru'), 'utf8');
const block = (text, name) => text.split(name + ' {\n')[1].split('\n}')[0];
const pre = block(collection, 'script:pre-request');
const post = block(collection, 'script:post-response');
const connection = (cursor, more, nodes = []) => ({
  pageInfo: { endCursor: cursor, hasNextPage: more },
  edges: nodes.map((node, index) => ({ cursor: `${cursor || 'last'}-${index}`, node }))
});
const product = (id, cursor = null, more = false) => ({
  id, images: connection(cursor, more, [{ id: id + '-image' }])
});
const catalog = (nodes, more = false, cursor = null) => ({
  products: { nodes, pageInfo: { hasNextPage: more, endCursor: cursor } }
});

function runner(file = '03-images.bru', limit = 0, followups = []) {
  const text = fs.readFileSync(path.join(root, file), 'utf8');
  const template = JSON.parse(block(text, 'body:json'));
  const name = text.match(/name: (.+)/)[1];
  const vars = new Map();
  let request, response, next, stopped;
  const context = {
    console: { log() {} },
    req: {
      getName: () => name,
      getBody: () => structuredClone(request || template),
      getUrl: () => 'https://example.test/graphql',
      setBody: value => { request = value; }
    },
    res: { getStatus: () => 200, getBody: () => response, setBody: body => { response = body; } },
    bru: {
      getVar: key => vars.get(key),
      setVar: (key, value) => { assert.match(key, /^[a-zA-Z0-9_.-]+$/); vars.set(key, value); },
      getEnvVar: key => key === 'catalogPageLimit' ? limit : undefined,
      getRequestVar: () => '15395172516091',
      sendRequest: async config => {
        assert.ok(followups.length, 'Unexpected continuation request');
        const next = followups.shift();
        next.check?.(config.data);
        return { status: 200, data: { data: next.data, extensions: { cost: { requestedQueryCost: 100 } } } };
      },
      setNextRequest: value => { next = value; },
      runner: { stopExecution: () => { stopped = true; } }
    }
  };
  return {
    vars,
    pre() {
      next = undefined; stopped = false; request = undefined;
      vm.runInNewContext(pre, { ...context });
      return JSON.parse(JSON.stringify(request || template));
    },
    body: () => response,
    async post(data, cost = 100, errors) {
      response = { data, extensions: { cost: { requestedQueryCost: cost } }, errors };
      await vm.runInNewContext("(async () => {\n" + post + "\n})()", { ...context });
      return { next, stopped };
    },
    stopped: () => stopped
  };
}

test('five-product pages drain independent cursors before advancing the catalogue', async () => {
  const r = runner();
  assert.equal(r.pre().variables.after, null);
  await r.post(catalog([product('a', 'a1', true), product('b'), product('c', 'c1', true), product('d'), product('e')], true, 'catalog1'));
  const request = r.pre();
  assert.deepEqual(request.variables, { id0: 'a', after0: 'a1', id1: 'c', after1: 'c1' });
  assert.match(request.query, /p0: product\(id:\$id0\)/);
  assert.match(request.query, /after:\$after1/);
  assert.ok(!request.query.includes('$itemAfter'));
  await r.post({ p0: product('a'), p1: product('c', 'c2', true) });
  assert.deepEqual(r.pre().variables, { id0: 'c', after0: 'c2' });
  await r.post({ p0: product('c') });
  assert.equal(r.pre().variables.after, 'catalog1');
  assert.equal((await r.post(catalog([product('f')]))).next, undefined);
  const result = r.vars.get('ankerCatalog.images.result');
  assert.equal(result.products, 6);
  assert.equal(result.complete, true);
  assert.equal(r.vars.get('ankerCatalog.images'), null);
  assert.equal(r.pre().variables.after, null);
});

test('page limit waits for all connection pages and reports partial catalogue', async () => {
  const r = runner('03-images.bru', 1);
  r.pre();
  await r.post(catalog([product('a', 'a1', true), ...['b','c','d','e'].map(id => product(id))], true, 'catalog1'));
  assert.equal(r.pre().variables.id0, 'a');
  assert.equal((await r.post({ p0: product('a') })).next, undefined);
  assert.equal(r.vars.get('ankerCatalog.images.result').complete, false);
});

test('empty catalogue ends successfully', async () => {
  const r = runner('01-details.bru');
  r.pre();
  assert.equal((await r.post(catalog([]))).next, undefined);
  assert.equal(r.vars.get('ankerCatalog.details.result').products, 0);
});

test('errors stop the runner and clear pagination state', async () => {
  for (const failure of ['cursor', 'null', 'nested', 'cost', 'graphql']) {
    const r = runner();
    r.pre();
    await r.post(catalog([product('a', 'a1', true)]));
    r.pre();
    let data = { p0: product('a') };
    if (failure === 'cursor') data.p0 = product('a', 'a1', true);
    if (failure === 'null') data.p0 = null;
    if (failure === 'nested') data.p0.images.edges[0].node.nested = connection('x', true);
    await assert.rejects(async () => r.post(data, failure === 'cost' ? 1001 : 100,
      failure === 'graphql' ? [{ message: 'Failure' }] : undefined));
    assert.equal(r.stopped(), true);
    assert.equal(r.vars.get('ankerCatalog.images'), null);
    assert.equal(r.vars.get('ankerCatalogValidated'), false);
  }
});

test('repeated catalogue cursors and product IDs fail', async () => {
  for (const repeat of ['cursor', 'id']) {
    const r = runner('07-removedFields.bru');
    r.pre();
    await r.post(catalog(['a','b','c','d','e'].map(id => ({id})), true, 'cursor1'));
    r.pre();
    const ids = repeat === 'id' ? ['a','g','h','i','j'] : ['f','g','h','i','j'];
    await assert.rejects(async () => r.post(catalog(ids.map(id => ({id})), true,
      repeat === 'cursor' ? 'cursor1' : 'cursor2')));
  }
});

test('invalid page limit fails before sending', async () => {
  for (const limit of [-1, 1.5, 'invalid']) {
    const r = runner('01-details.bru', limit);
    assert.throws(() => r.pre());
    assert.equal(r.stopped(), true);
  }
});

test('details fetches and validates fifty products per page', async () => {
  const r = runner('01-details.bru', 1);
  assert.match(r.pre().query, /products\(first:50,/);
  await r.post(catalog(Array.from({length:50}, (_,i) => ({id:String(i)})), true, 'next'));
  assert.equal(r.vars.get('ankerCatalog.details.result').products, 50);
  const invalid = runner('01-details.bru');
  invalid.pre();
  await assert.rejects(() => invalid.post(catalog([{id:'a'}], true, 'next')), /page size/);
});

test('single merges top-level and nested pages into one product response', async () => {
  const id = 'gid://shopify/Product/15395172516091';
  const followups = [
    { data: { product: { id, images: connection('image2', false, [{ id: 'image2' }]) } } },
    {
      check(body) {
        assert.match(body.query, /variants\(first:1,after:null\)/);
        assert.match(body.query, /sellingPlanAllocations\(first:1,after:"plan1"\)/);
      },
      data: { product: { id, variants: connection('v1', false, [{
        id: 'v1', sellingPlanAllocations: connection('plan2', false, [{ sellingPlan: { id: 'plan2' } }])
      }]) } }
    }
  ];
  const r = runner('single.bru', 0, followups);
  assert.equal(r.pre().variables.id, id);
  await r.post({ product: {
    id, title: 'Product', images: connection('image1', true, [{ id: 'image1' }]),
    variants: connection('v1', false, [{
      id: 'v1', sellingPlanAllocations: connection('plan1', true, [{ sellingPlan: { id: 'plan1' } }])
    }])
  } });
  assert.equal(followups.length, 0);
  const p = r.body().data.product;
  assert.equal(p.title, 'Product');
  assert.equal(p.images.edges.length, 2);
  assert.equal(p.variants.edges[0].node.sellingPlanAllocations.edges.length, 2);
  assert.equal(r.body().extensions.pagination.requests, 3);
  assert.equal(r.vars.get('ankerCatalogValidated'), true);
});

test('prices stops at 500 products and drains variants from both catalogue pages', async () => {
  const products = Array.from({ length: 500 }, (_, i) => ({
    id: `p${i}`, availableForSale: true,
    variants: connection('v5', i === 0 || i === 499, [{ id: 'v1', availableForSale: true, price: { amount: '1', currencyCode: 'AUD' } }])
  }));
  const followups = [{
    check(body) {
      assert.equal(body.variables.after, 'page1');
      assert.match(body.query, /products\(first:250,after:\$after,sortKey:ID\)/);
    },
    data: catalog(products.slice(250), true, 'page2')
  }, { data: { product: {
    id: 'p0', variants: connection('v6', false, [{ id: 'v6', availableForSale: false, price: { amount: '6', currencyCode: 'AUD' } }])
  } } }, { data: { product: {
    id: 'p499', variants: connection('v6', false, [{ id: 'v6', availableForSale: true, price: { amount: '6', currencyCode: 'AUD' } }])
  } } }];
  const r = runner('08-availability-prices.bru', 0, followups);
  assert.equal(r.pre().variables.after, null);
  await r.post(catalog(products.slice(0, 250), true, 'page1'));
  assert.equal(followups.length, 0);
  assert.equal(r.body().data.products.nodes.length, 500);
  assert.equal(r.body().data.products.pageInfo.hasNextPage, true);
  assert.equal(r.body().data.products.pageInfo.endCursor, 'page2');
  assert.equal(r.body().data.products.nodes[0].variants.edges.length, 2);
  assert.equal(r.body().data.products.nodes[0].variants.edges[1].node.availableForSale, false);
  assert.equal(r.body().data.products.nodes[499].variants.edges.length, 2);
});

test('prices returns fewer than 500 when the catalogue ends', async () => {
  for (const count of [0, 146, 267]) {
    const products = Array.from({ length: count }, (_, i) => ({ id: `p${i}`, variants: connection(null, false) }));
    const followups = count > 250 ? [{ data: catalog(products.slice(250), false, 'last') }] : [];
    const r = runner('08-availability-prices.bru', 0, followups);
    r.pre();
    await r.post(catalog(products.slice(0, 250), count > 250, 'page1'));
    assert.equal(followups.length, 0);
    assert.equal(r.body().data.products.nodes.length, count);
    assert.equal(r.body().data.products.pageInfo.hasNextPage, false);
  }
});

test('prices rejects repeated catalogue cursors and duplicate product IDs across pages', async () => {
  for (const failure of ['cursor', 'id']) {
    const products = Array.from({ length: 250 }, (_, i) => ({ id: `p${i}` }));
    const next = Array.from({ length: 250 }, (_, i) => ({ id: `p${i + (failure === 'id' ? 0 : 250)}` }));
    const r = runner('08-availability-prices.bru', 0, [{
      data: catalog(next, true, failure === 'cursor' ? 'page1' : 'page2')
    }]);
    r.pre();
    await assert.rejects(() => r.post(catalog(products, true, 'page1')), /repeated/i);
    assert.equal(r.stopped(), true);
    assert.equal(r.vars.get('ankerCatalogValidated'), false);
  }
});

test('merged requests reject absent products and repeated connection cursors', async () => {
  const missing = runner('single.bru');
  missing.pre();
  await assert.rejects(() => missing.post({ product: null }), /Missing/);
  assert.equal(missing.vars.get('ankerCatalogValidated'), false);
  const id = 'gid://shopify/Product/15395172516091';
  const p = { id, images: connection('same', true, [{ id: 'image' }]) };
  const repeated = runner('single.bru', 0, [{ data: { product: structuredClone(p) } }]);
  repeated.pre();
  await assert.rejects(() => repeated.post({ product: p }), /Repeated pagination/);
  assert.equal(repeated.stopped(), true);
  assert.equal(repeated.vars.get('ankerCatalogValidated'), false);
});
