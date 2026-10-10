// Puente con el núcleo en Rust. Los tipos reflejan `archito-core/src/model.rs`.

import { invoke } from "@tauri-apps/api/core";

export type Level = "safe" | "confirm" | "danger";
export type Status = "ok" | "warn" | "error" | "bye";

export interface Action {
  id: number;
  label: string;
  command: string;
  level: Level;
  description: string;
  danger_note: string;
}

export interface Step {
  token: string;
  doc: string;
}

export interface Stat {
  label: string;
  pct: number;
  value: string;
  severity: "good" | "warn" | "danger";
}

export interface Reply {
  text: string;
  code: string;
  steps: Step[];
  actions: Action[];
  chips: string[];
  stats: Stat[];
  learn: { command: string; note: string } | null;
  status: Status;
  view: "" | "camino" | "bitacora";
}

export interface ExecResult {
  ok: boolean;
  mode: "inline" | "terminal";
  output: string;
}

export interface Mission {
  id: string;
  stage: string;
  title: string;
  why: string;
  steps: string[];
  command: string;
  xp: number;
  state: "done" | "active" | "open";
}

export interface PathView {
  level: number;
  level_name: string;
  xp: number;
  next_xp: number | null;
  done: number;
  total: number;
  missions: Mission[];
  hook: boolean;
}

export interface CheckResult {
  done: boolean;
  message: string;
  xp: number;
  level_up: string | null;
}

export interface LogEntry {
  program: string;
  doc: string;
  times: number;
}

export interface Launch {
  view: "" | "camino" | "bitacora";
  text: string;
}

export const api = {
  launchRequest: () => invoke<Launch>("launch_request"),
  framePainted: () => invoke<void>("frame_painted"),
  welcome: () => invoke<Reply>("welcome"),
  ask: (text: string) => invoke<Reply>("ask", { text }),
  explain: (command: string) => invoke<Reply>("explain", { command }),
  runAction: (id: number) => invoke<ExecResult>("run_action", { id }),
  cancelAction: (id: number) => invoke<void>("cancel_action", { id }),
  path: () => invoke<PathView>("path"),
  startMission: (id: string) => invoke<PathView>("start_mission", { id }),
  checkMission: (id: string) => invoke<CheckResult>("check_mission", { id }),
  logbook: () => invoke<LogEntry[]>("logbook"),
  setRegion: (x: number, y: number, w: number, h: number) => invoke<void>("set_region", { x, y, w, h }),
  setExpanded: (expanded: boolean) => invoke<void>("set_expanded", { expanded }),
};
