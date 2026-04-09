export type CommandStatus = 'pending' | 'queued_for_poll' | 'sent' | 'failed';

export interface QueuedCommand {
  id: string;
  nodeId: string;
  payload: string;
  kind: string;
  createdAt: string;
  status: CommandStatus;
}
