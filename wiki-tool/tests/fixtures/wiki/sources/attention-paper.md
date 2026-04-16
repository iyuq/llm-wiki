---
title: "Attention Is All You Need"
type: source
tags: [transformer, attention, deep-learning]
sources: [attention-paper]
last_updated: 2026-04-16
---

# Attention Is All You Need

Summary of the seminal paper introducing the [[Transformer]] architecture.

The paper proposes dispensing with recurrence and convolutions entirely,
relying solely on [[Self-Attention]] mechanisms for sequence transduction.

## Key Ideas

- [[Multi-Head Attention]] allows the model to attend to information from
  different representation subspaces at different positions.
- Positional encoding injects sequence order information without recurrence.
- The architecture achieves state-of-the-art on WMT 2014 English-to-German
  and English-to-French translation tasks.

## Authors

[[Ashish Vaswani]], [[Noam Shazeer]], and others at Google Brain and
Google Research.
