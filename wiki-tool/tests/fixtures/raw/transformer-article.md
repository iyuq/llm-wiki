# Understanding Transformer Architecture

Transformers are a type of neural network architecture introduced in the paper "Attention Is All You Need" by Vaswani et al. in 2017.

## Key Concepts

The transformer architecture relies on **self-attention mechanisms** to process input sequences in parallel, unlike recurrent neural networks (RNNs) which process tokens sequentially.

### Multi-Head Attention

Multi-head attention allows the model to jointly attend to information from different representation subspaces at different positions. It uses queries, keys, and values computed from the input.

### Positional Encoding

Since transformers don't have built-in sequence order awareness, positional encodings are added to the input embeddings to provide position information.

## Impact

The transformer architecture has been foundational to:

- **GPT** (Generative Pre-trained Transformer) by OpenAI
- **BERT** (Bidirectional Encoder Representations from Transformers) by Google
- **T5** (Text-to-Text Transfer Transformer) by Google

These models have achieved state-of-the-art results across many NLP tasks including translation, summarization, and question answering.

## 注意力机制

注意力机制是transformer架构的核心组件。它允许模型在处理序列时动态地关注不同位置的信息。
