-- Build state tracking for release orchestration
CREATE TABLE IF NOT EXISTS build_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  release_version TEXT NOT NULL,
  repo_name TEXT NOT NULL,
  workflow_run_id INTEGER,
  status TEXT NOT NULL DEFAULT 'pending', -- pending, in_progress, success, failure, cancelled, timed_out
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  error_message TEXT,
  retry_count INTEGER DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(release_version, repo_name)
);

-- Build state transitions (audit log)
CREATE TABLE IF NOT EXISTS build_state_transitions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  build_run_id INTEGER NOT NULL REFERENCES build_runs(id),
  from_status TEXT,
  to_status TEXT NOT NULL,
  transitioned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  reason TEXT
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_build_runs_release ON build_runs(release_version);
CREATE INDEX IF NOT EXISTS idx_build_runs_status ON build_runs(status);
CREATE INDEX IF NOT EXISTS idx_build_runs_repo ON build_runs(repo_name);
CREATE INDEX IF NOT EXISTS idx_build_transitions_build ON build_state_transitions(build_run_id);
CREATE INDEX IF NOT EXISTS idx_build_transitions_time ON build_state_transitions(transitioned_at DESC);

