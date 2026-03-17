/**
 * Cache Info
 *
 * Machinery for tracking streaming metadata about Flight prerender responses.
 *
 * During a prerender, certain information (like the cache stage level) is not
 * known until rendering completes. This module provides thenables that follow
 * the React Flight thenable protocol: they are embedded in the Flight payload
 * before rendering begins, then resolved right before the prerender is aborted.
 * Flight serializes the resolved values lazily into the response stream.
 *
 * This pattern is only used for prerender responses (not dynamic requests or
 * navigations), because only prerenders have the two-phase lifecycle where the
 * payload is constructed before the final values are known.
 *
 * TODO: Vary params (see vary-params.ts) use the same pattern — a thenable
 * embedded in the response, resolved after rendering, serialized by Flight.
 * The shared parts of that implementation should be unified into this module
 * so each new piece of cache info doesn't need its own thenable type, create
 * function, and resolve function.
 */

import type { CacheInfoThenable } from '../../shared/lib/segment-cache/cache-info-decoding'

/**
 * Server-side CacheInfoThenable with additional fields for accumulation
 * during rendering. Extends the shared CacheInfoThenable with:
 * - `current`: mutable accumulator updated during rendering (inspired by
 *   React refs). Used to derive the final `value` when resolved.
 * - `resolvers`: callbacks waiting for resolution.
 */
export type ServerCacheInfoThenable<T> = CacheInfoThenable<T> & {
  status: 'pending' | 'fulfilled'
  value: T
  current: T
  resolvers: Array<(value: T) => void>
}

/**
 * Creates a pending ServerCacheInfoThenable with the given default value.
 * Flight serializes it lazily into the response stream.
 */
export function createCacheInfoThenable<T>(
  defaultValue: T
): ServerCacheInfoThenable<T> {
  const thenable = {
    status: 'pending' as const,
    value: defaultValue,
    current: defaultValue,
    then(onfulfilled: ((value: T) => unknown) | null | undefined) {
      if (onfulfilled) {
        if (thenable.status === 'pending') {
          thenable.resolvers.push(onfulfilled)
        } else {
          onfulfilled(thenable.value)
        }
      }
    },
    resolvers: [] as Array<(value: T) => void>,
  } as ServerCacheInfoThenable<T>
  return thenable
}

/**
 * Resolves a ServerCacheInfoThenable with its final value.
 */
export function resolveCacheInfoThenable<T>(
  thenable: ServerCacheInfoThenable<T>,
  value: T
): void {
  if (thenable.status !== 'pending') {
    return
  }
  thenable.value = value
  thenable.status = 'fulfilled'
  for (const resolver of thenable.resolvers) {
    resolver(value)
  }
  thenable.resolvers = []
}
