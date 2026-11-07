# Experiments

One row per experiment branch. Experiment branches (`exp/*`) are never merged; a winner is redone on a normal task branch and goes through a pull request.

Before the first row, run the unmodified code through the oracle once and record it: an oracle that already gives the baseline a full score, or whose run-to-run noise exceeds the gain you hope for, cannot judge a change.

| Date | Branch | Hypothesis | Oracle (command, environment, baseline) | Result vs baseline | Submitted as (`cand/` tag) and score | Decision |
|---|---|---|---|---|---|---|
| <YYYY-MM-DD> | `exp/<id>-<slug>` | <what should get faster or better, and why> | `scripts/bench-ab.sh <base> <candidate>` on <machine> | <numbers> | `cand/<id>-<n>`: <score>, or "not submitted" | keep → <task id> / drop |
