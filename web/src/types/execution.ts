export type ExecutionState =
  | "IDLE"
  | "RUNNING"
  | "COMPLETED"
  | "FAILED"
  | "CANCELLED";

export const EventType = {
  STARTED: 0,
  STDOUT: 1,
  STDERR: 2,
  FINISHED: 3,
} as const;

// Extract the values as a type (0 | 1 | 2 | 3)
export type EventType = (typeof EventType)[keyof typeof EventType];

export interface StreamMessage {
  type: EventType;
  data?: string; // Base64 encoded string from Go
}
