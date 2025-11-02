-- Migration 008: Signature Reasoning
-- Adds reasoning field to signatures (signatures are stored as JSON in pull_requests.signatures)
-- This migration is informational - no schema changes needed as signatures are JSON
-- The reasoning will be stored as part of the signature object in the JSON array

