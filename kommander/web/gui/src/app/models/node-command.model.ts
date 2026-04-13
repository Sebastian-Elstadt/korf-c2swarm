/** Matches `NodeCommandEntryItem` in `kommander/web/src/api/nodes.rs`. */
export interface NodeCommandEntry {
  id: string;
  created_at: string;
  status: number;
  command_type: number;
  last_attempted_at: string | null;
  completed_at: string | null;
  text_content: string | null;
}

/** Matches `AddNodeCommandRequest` (POST body). */
export interface AddNodeCommandRequest {
  command_type: number;
  text_content?: string | null;
}
