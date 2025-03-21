# MCP Core Library

This crate contains the domain business logic and ports of the MCP server following a hexagonal architecture.

## Architectural Principles

Following the principles of hexagonal architecture, this crate:

- Contains only the **domain business logic** (pure logic) and **ports** (interfaces)
- Defines abstractions that represent domain boundaries
- Remains independent of implementation details and specific technologies

## Main Components

- **Primary/driving ports**: Interfaces used by input adapters
  - `Tool`: Port defining the contract for tools executable via MCP

- **Secondary/driven ports**: Interfaces used by the domain to communicate with the outside world
  - `Transport`: Port defining the contract for network communication

- **Domain entities and types**: Fundamental domain data structures
  - Abstract message types (independent of specific formats like JSON-RPC)
  - Domain errors
  - Tool registry (`ToolRegistry`)

## Note on Serialization

Specific protocol details like JSON-RPC are deliberately excluded from this domain crate and will be implemented in external adapters, in accordance with hexagonal architecture principles.

This crate only depends on basic crates like `serde` and `tokio` (for async types) to remain minimal and preserve domain isolation.

## Roadmap

In addition to the main traits, the core crate defines other interfaces or structures for domain organization, such as `ToolRegistry` to manage the registration and search of tools by name. For immediate needs, a simple implementation internal to the core (with a HashMap<String, Box<dyn Tool>>) is sufficient, but could be formalized as a trait if different storage or loading strategies become necessary (e.g., for dynamic plugins).