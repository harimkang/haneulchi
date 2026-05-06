import type { TerminalPtyOutputEvent } from "../services/terminalPtyClient";

export interface TerminalTransportSessionState {
  nextSeq: number;
  pending: Record<number, string>;
  droppedChunks: number;
}

export interface TerminalTransportState {
  sessions: Record<string, TerminalTransportSessionState>;
  maxPendingPerSession: number;
}

export interface TerminalTransportIngestResult {
  state: TerminalTransportState;
  releasedChunks: string[];
}

interface CreateTerminalTransportStateOptions {
  maxPendingPerSession?: number;
}

export function createTerminalTransportState(options: CreateTerminalTransportStateOptions = {}): TerminalTransportState {
  return {
    sessions: {},
    maxPendingPerSession: options.maxPendingPerSession ?? 256,
  };
}

export function ingestTerminalPtyOutput(
  state: TerminalTransportState,
  event: TerminalPtyOutputEvent,
): TerminalTransportIngestResult {
  const current = state.sessions[event.sessionId] ?? { nextSeq: 1, pending: {}, droppedChunks: 0 };

  if (event.seq < current.nextSeq || current.pending[event.seq] !== undefined) {
    return { state, releasedChunks: [] };
  }

  const pending = {
    ...current.pending,
    [event.seq]: event.chunk,
  };
  const releasedChunks: string[] = [];
  let nextSeq = current.nextSeq;

  while (pending[nextSeq] !== undefined) {
    releasedChunks.push(pending[nextSeq]);
    delete pending[nextSeq];
    nextSeq += 1;
  }
  const bounded = boundPendingChunks(pending, state.maxPendingPerSession);

  return {
    state: {
      maxPendingPerSession: state.maxPendingPerSession,
      sessions: {
        ...state.sessions,
        [event.sessionId]: {
          nextSeq,
          pending: bounded.pending,
          droppedChunks: current.droppedChunks + bounded.droppedChunks,
        },
      },
    },
    releasedChunks,
  };
}

function boundPendingChunks(
  pending: Record<number, string>,
  maxPendingPerSession: number,
): { pending: Record<number, string>; droppedChunks: number } {
  const seqs = Object.keys(pending).map(Number).sort((a, b) => a - b);
  if (seqs.length <= maxPendingPerSession) {
    return { pending, droppedChunks: 0 };
  }

  const keep = new Set(seqs.slice(0, maxPendingPerSession));
  const bounded: Record<number, string> = {};
  for (const seq of keep) {
    bounded[seq] = pending[seq];
  }

  return {
    pending: bounded,
    droppedChunks: seqs.length - keep.size,
  };
}
