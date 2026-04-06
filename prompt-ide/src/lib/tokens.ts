import { encode } from 'gpt-tokenizer';
import type { ModelConfig } from './types';

export function countTokens(text: string): number {
  if (!text) return 0;
  try {
    return encode(text).length;
  } catch {
    // Fallback: rough estimate ~4 chars per token
    return Math.ceil(text.length / 4);
  }
}

export function estimateCost(
  tokensIn: number,
  tokensOut: number,
  model: ModelConfig
): number {
  return (tokensIn / 1000) * model.inputCostPer1k + (tokensOut / 1000) * model.outputCostPer1k;
}

export function formatCost(cost: number): string {
  if (cost < 0.001) return `$${cost.toFixed(6)}`;
  if (cost < 0.01) return `$${cost.toFixed(4)}`;
  return `$${cost.toFixed(3)}`;
}

export function formatTokens(count: number): string {
  if (count < 1000) return count.toString();
  return `${(count / 1000).toFixed(1)}k`;
}
