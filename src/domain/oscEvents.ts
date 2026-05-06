export interface OscEvent {
  code: string;
  payload: string;
}

export interface OscEventState {
  pending: string;
  allowedEvents: number;
  ignoredEvents: number;
  rejectedEvents: number;
  maxPayloadBytes: number;
}

interface CreateOscEventStateOptions {
  maxPayloadBytes?: number;
}

export interface ParseOscSequencesResult {
  state: OscEventState;
  events: OscEvent[];
  displayText: string;
}

const allowedOscCodes = new Set(["9", "99", "777"]);
const oscStart = "\u001b]";
const bellTerminator = "\u0007";
const stTerminator = "\u001b\\";

export function createOscEventState(options: CreateOscEventStateOptions = {}): OscEventState {
  return {
    pending: "",
    allowedEvents: 0,
    ignoredEvents: 0,
    rejectedEvents: 0,
    maxPayloadBytes: options.maxPayloadBytes ?? 4096,
  };
}

export function parseOscSequences(state: OscEventState, chunk: string): ParseOscSequencesResult {
  const input = state.pending + chunk;
  const events: OscEvent[] = [];
  let displayText = "";
  let cursor = 0;
  let pending = "";
  let allowedEvents = state.allowedEvents;
  let ignoredEvents = state.ignoredEvents;
  let rejectedEvents = state.rejectedEvents;

  while (cursor < input.length) {
    const start = input.indexOf(oscStart, cursor);
    if (start === -1) {
      displayText += input.slice(cursor);
      break;
    }

    displayText += input.slice(cursor, start);
    const terminator = findOscTerminator(input, start + oscStart.length);
    if (!terminator) {
      pending = input.slice(start);
      break;
    }

    const rawPayload = input.slice(start + oscStart.length, terminator.index);
    const event = parseOscPayload(rawPayload);
    if (!allowedOscCodes.has(event.code)) {
      ignoredEvents += 1;
    } else if (payloadByteLength(event.payload) > state.maxPayloadBytes) {
      rejectedEvents += 1;
    } else {
      allowedEvents += 1;
      events.push(event);
    }

    cursor = terminator.index + terminator.length;
  }

  return {
    state: {
      pending,
      allowedEvents,
      ignoredEvents,
      rejectedEvents,
      maxPayloadBytes: state.maxPayloadBytes,
    },
    events,
    displayText,
  };
}

function findOscTerminator(input: string, from: number): { index: number; length: number } | undefined {
  const bell = input.indexOf(bellTerminator, from);
  const st = input.indexOf(stTerminator, from);

  if (bell === -1 && st === -1) return undefined;
  if (bell === -1) return { index: st, length: stTerminator.length };
  if (st === -1) return { index: bell, length: bellTerminator.length };
  return bell < st ? { index: bell, length: bellTerminator.length } : { index: st, length: stTerminator.length };
}

function parseOscPayload(rawPayload: string): OscEvent {
  const separator = rawPayload.indexOf(";");
  if (separator === -1) {
    return {
      code: rawPayload,
      payload: "",
    };
  }

  return {
    code: rawPayload.slice(0, separator),
    payload: rawPayload.slice(separator + 1),
  };
}

function payloadByteLength(payload: string): number {
  return new TextEncoder().encode(payload).length;
}
