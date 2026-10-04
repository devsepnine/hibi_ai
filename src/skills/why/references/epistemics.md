# Epistemics

Every claim in a `why` answer sits in one of these tiers. The tier decides which output section the claim goes in and how it is phrased. Adapted from pstack's `why` epistemics.

| Tier | What qualifies | Phrasing | Section |
|---|---|---|---|
| **Direct** | Someone wrote down the reason: a PR body, a ticket, a code comment, a design doc, a chat message from the author | "This exists because X", with the citation | What we found |
| **Supported** | Several indirect signals agree, but no single source states it | "The evidence points to X: A, B, and C", citing each | What we found |
| **Inferred** | A reasonable reading of the context that nothing states | "It appears", "likely", with the chain: "Given A and B, C seems likely because D" | What we can infer |
| **Speculative** | Plausible, but other explanations fit as well | "One possibility is X, but we found no evidence" | Competing readings |
| **Unknown** | Searched and found nothing | "We searched X with terms A and B and found no rationale" | What we don't know |

## Rules

- Never promote a tier while summarizing. Two inferences that agree are still inferences unless the signals behind them are independent.
- "The code does X, so the author wanted X" is an inference, not a Direct claim.
- Be specific about the null. "We couldn't find out" helps less than "we read the 6 PRs that touched this file since 2023 and searched Jira for the constant's value; none gave a reason".
- Prefer the older source when it states the reason and the newer one only restates it.
- These tiers refine the measured, inferred, and guess labels in `CLAUDE.md`: Direct claims count as measured, Supported and Inferred as inferred, and Speculative as guess.
