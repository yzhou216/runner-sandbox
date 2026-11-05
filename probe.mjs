// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Review a sample diff with OpenRabbit's own prompt and LLM client, as the
// review workflow does, but without a pull request.  The diff turns addition
// into subtraction, which a working review should flag.
import { createLLMClient } from './openrabbit/dist/llm/index.js';
import { buildReviewPrompt } from './openrabbit/dist/reviewer.js';

const client = createLLMClient('openrouter', {
  apiKey: process.env.LLM_API_KEY,
  apiUrl: 'https://openrouter.ai/api/v1',
  model: 'openrouter/free',
});

const review = await client.complete(buildReviewPrompt({
  title: 'Simplify add',
  body: 'Tidy the arithmetic in add.',
  changedFiles: [{
    path: 'src/math.js',
    patch: '@@ -1,3 +1,3 @@\n export function add(a, b) {\n-  return a + b;\n+  return a - b;\n }\n',
  }],
  reviewMode: 'both',
  toneMode: 'balanced',
}));

console.log(JSON.stringify({
  verdict: review.summary.verdict,
  overview: review.summary.overview,
  comments: review.comments.map(({ path, line, type, body }) => ({ path, line, type, body })),
}, null, 2));
if (!review.summary.overview && review.comments.length === 0) {
  process.exit(1);
}
