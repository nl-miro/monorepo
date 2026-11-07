
GOAL: missing data from offer details 

- [x] Build the ignored `tmp/` retailer API downloader, reusable JSON requests, resumable pagination, and local fixture tests.
- [~] Finish downloading public Anker, Minisforum, and Ugreen product API data for every supported market; inspect `python3 tmp/status.py --run full-20261107` for progress and coverage gaps.



ankerau
- after extraction, we are missing `productInfos` `subtitle`
- after extraction, we are missing `components` `htmlContainer`
- structuralized is missing colors


































Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase                      |
| ------ | -------------------------- |
| `[ ]`  | Pending                    |
| `[.]`  | Planning in progress       |
| `[>]`  | Planning complete          |
| `[~]`  | Implementation in progress |
| `[x]`  | Implemented                |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.
