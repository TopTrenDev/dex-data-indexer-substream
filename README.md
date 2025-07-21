# dex_data_parser Substreams modules

This package was initialized via `substreams init`, using the `sol-hello-world` template.

## Usage

```bash
substreams build
substreams auth
substreams gui       			  # Get streaming!
```

Optionally, you can publish your Substreams to the [Substreams Registry](https://substreams.dev).

```bash
substreams registry login         # Login to substreams.dev
substreams registry publish       # Publish your Substreams to substreams.dev
```

## Modules

### `map_filtered_transactions`

This module retrieves Solana transactions filtered by one or several Program IDs.
You will only receive transactions containing the specified Program IDs).

**NOTE:** Transactions containing voting instructions will NOT be present.

### `map_my_data`

This module allows you to create transformations on the filtered transactions.
