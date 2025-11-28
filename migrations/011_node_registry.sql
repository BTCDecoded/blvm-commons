-- Migration: External Node Registry
-- Simple registry for nodes/miners to register for fee forwarding attribution

CREATE TABLE IF NOT EXISTS node_registry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL UNIQUE,  -- Unique identifier (e.g., "node-001", "miner-pool-abc")
    node_name TEXT NOT NULL,  -- Human-readable name
    node_type TEXT NOT NULL,  -- 'miner', 'node', 'pool', 'exchange', etc.
    bitcoin_addresses TEXT,  -- JSON array of Bitcoin addresses this node controls
    registered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    active BOOLEAN DEFAULT TRUE,
    metadata TEXT  -- JSON metadata (contact info, etc.)
);

CREATE INDEX IF NOT EXISTS idx_node_id ON node_registry(node_id);
CREATE INDEX IF NOT EXISTS idx_node_active ON node_registry(active);
CREATE INDEX IF NOT EXISTS idx_node_type ON node_registry(node_type);

-- Table to map Bitcoin addresses to node IDs for fast lookup
CREATE TABLE IF NOT EXISTS address_to_node (
    address TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES node_registry(node_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_address_node ON address_to_node(node_id);

