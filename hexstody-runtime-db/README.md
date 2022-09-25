This package provides runtime state, shared between hexstody-public and hexstody-operator

The state is clear after restarts

At the moment (25.09) it tracks challenges for key-based auth and tickers for currencies and fiats.

When a ticker is requested, if it is cached, it is returned from cache. If it is not present in the cache, the getter requests the ticker from ticker provider and caches it.