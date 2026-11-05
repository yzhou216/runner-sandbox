<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# runner-sandbox: review key

Checked the `LLM_API_KEY` secret that `review.yml` passes to OpenRabbit,
without a pull request, on `ubuntu-latest`.  `key.yml` asks OpenRouter about
the key, then has `probe.mjs` review a sample diff with OpenRabbit's own
prompt and LLM client, built from commit `0350c5f` of its `main` branch with
the `openrouter/free` model.

Outcome, in run 42072352162: the job passed.

- `GET https://openrouter.ai/api/v1/key` returned HTTP 200 with
  `is_free_tier` true.
- The review took 13 seconds.  Its verdict was "needs changes", and its one
  inline comment, on line 3 of `src/math.js`, called the change from `a + b`
  to `a - b` a bug.
- The log shows the secret only as `***`.

Not covered: posting a review to a pull request, which needs the
`pull_request_target` event.
