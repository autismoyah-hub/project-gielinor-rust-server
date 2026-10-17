# Pinned upstream and publication ledger

| Role | Source SHA | Status |
|---|---|---|
| Initial/common server engine | 6a1ea78022bd1c9cbc7e259bcb3595652e5eaae5 | Tested independent baseline/common ancestor |
| Earlier observed Agent 2 ref | 924e929438c952d994287fad6f0c0bce749438ce | Historical observation, not merged |
| Final observed Agent 2 ref | 521e2e047071a7e7d481ab458829bb88a57b76e2 | Read-only fetch/review; chemistry/queue changes and V12 collision documented |
| Native client baseline | 95eed88159b06cebea24559a814a5111b54a59cd | Independent reference clone |
| Final local validation adapter | a32b66deeeb30a2be159fbd379a9ffb07f423b72 | Source-only seven-commit patch; client push disabled |
| Published foundation | 311d2f8150764e984012bad9c6d1bad55b1671a0 | Verified remote historical checkpoint |
| Durable checkpoint milestone | 0e2c54c7afd8ac1b097bf973dfce44a786bf0f82 | Verified remote historical checkpoint |
| Accepted gameplay implementation | 420729798ffc3b0d31c7aff7497d1841597c2104 | Final tested server gameplay source; later final-report commit adds metadata/controllers/client validation patch only |

No external gameplay source is overlaid onto either clone. Direction B deliberately replaces prototype shell/progression; it is not claimed bit-identical to the historical Kotlin oracle. Integration into newer Agent 2 code is explicitly separate and must resolve the V12 migration number collision.
