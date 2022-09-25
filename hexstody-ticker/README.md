Ticker API

Contains two modules:
    * **api** implements ticker API and provides `Vec<Route>` to mount in both `hexstody-operator` and `hexstody-public`. This API is public, no authorization is provided
    * **worker**  implements ticker worker, that updates runtime cache of tickers. It only updates currently tracked tickers, since `RuntimeState` independently requests missing ticker from ticker provider. If it was requested at some point, it will be updated. 