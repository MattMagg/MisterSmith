# MS-77 Bounded External-Agent Surface

Date: 2026-03-19
Issue: `MS-77`
Status: complete

## Objective

Make one external-agent surface explicitly discoverable without widening Mister Smith into a new
mesh protocol or bypassing the zero-trust delegation substrate.

## Chosen surface

Use the existing MCP server boundary as the bounded external-agent surface.

Reason:

- MCP tool invocation already preserves zero-trust delegation checks before the handler executes.
- External agents already use `tools/list` and `tools/call` on this boundary.
- The remaining gaps were explicit capability discovery and an operator-visible way to inspect the
  same boundary without a second transport.

## What changed

- `crates/mister-smith-mcp/src/server.rs`
  - exposed a typed `ExternalCapabilityDescriptor` for MCP tools
  - published descriptor metadata for each listed tool
  - tightened invocation enforcement from descriptor-only validation to exact delegated action
    matching at the MCP boundary
- `crates/mister-smith-mcp/src/client.rs`
  - preserved the published capability metadata during tool discovery
  - exposed the metadata on discovered `McpTool` entries instead of dropping it
- `crates/mister-smith-mcp/src/bridge.rs`
  - updated bridge tests for the richer `McpTool` shape
- `crates/mister-smith-mcp/src/compatibility.rs`
  - added `describe_external_capabilities` as a bounded operator-facing discovery tool
  - published the registered capability catalog through the existing Smith compatibility surface
  - required a delegated `Discover` action for that inspection path

## Boundary proof

For each listed MCP tool, the server now publishes a stable capability description with:

- boundary family: `mcp.tool`
- external name
- descriptor id
- action id
- required scope
- namespace
- revocation key

This keeps discovery and execution aligned:

- discovery tells the external agent exactly which descriptor is required
- execution rejects mismatched, revoked, or wrong-action delegated authority at the MCP boundary
- operators can inspect the same catalog through `describe_external_capabilities` without widening
  the boundary beyond MCP

## Validation

- `cargo test -p mister-smith-mcp`
- `cargo build --workspace`
- `git diff --check`

## Result

`MS-77` now has one bounded external-agent surface with explicit capability discovery on the
existing zero-trust MCP boundary plus one bounded inspection tool on the same surface.

Non-goals intentionally unchanged:

- no full A2A or multi-protocol mesh work
- no new ambient-trust path
- no HTTP-side protocol expansion
