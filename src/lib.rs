use async_trait::async_trait;
use serde_json;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Transport port for exchanging serialized MCP messages.
#[async_trait]
pub trait Transport {
    /// Reads the next request (as JSON text) from the channel.
    async fn receive(&mut self) -> Result<String, TransportError>;
    /// Sends a JSON message (response or notification) to the channel.
    async fn send(&mut self, message: &str) -> Result<(), TransportError>;
}

/// Transport error management port.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Error when reading data
    #[error("Read Error: {0}")]
    ReadError(String),
    
    /// Error when writing data
    #[error("Write Error: {0}")]
    WriteError(String),
    
    /// Connection error
    #[error("Connection Error: {0}")]
    ConnectionError(String),
    
    /// JSON serialization/deserialization error
    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Abstract domain message types, independent of specific protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Tool execution request
    Request(Request),
    /// Response to a request
    Response(Response),
    /// Notification (message without expected response)
    Notification(Notification),
}

/// Tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Unique request identifier
    pub id: String,
    /// Name of the tool to execute
    pub tool_name: String,
    /// Tool parameters (content depends on the tool)
    pub params: serde_json::Value,
}

/// Response to a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Identifier corresponding to the request
    pub id: String,
    /// Execution result or error
    pub result: Result<serde_json::Value, Error>,
}

/// Notification (message without expected response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Event name
    pub event: String,
    /// Data associated with the notification
    pub params: serde_json::Value,
}

/// Domain error for responses
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum Error {
    /// Error related to a specific tool
    #[error("Tool Error: {0}")]
    ToolError(String),
    
    /// Tool not found
    #[error("Tool Not Found: {0}")]
    ToolNotFound(String),
    
    /// Invalid parameters
    #[error("Invalid Parameters: {0}")]
    InvalidParams(String),
    
    /// Internal error
    #[error("Internal Error: {0}")]
    InternalError(String),
}

/// Tool port executing an action requested via MCP.
#[async_trait]
pub trait Tool {
    /// Executes the tool with the provided parameters (as JSON).
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError>;
    // Eventually, we can include a method to describe the input/output schema
}

/// Tool error management port.
#[derive(Debug, Error)]
pub enum ToolError {
    /// Invalid parameters provided to the tool
    #[error("Invalid Parameters: {0}")]
    InvalidParams(String),
    
    /// The tool encountered an error during execution
    #[error("Execution Error: {0}")]
    ExecutionError(String),
    
    /// The requested tool does not exist
    #[error("Tool Not Found: {0}")]
    ToolNotFound(String),
    
    /// JSON serialization/deserialization error
    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Type representing a registry of available tools
pub type ToolRegistry = std::collections::HashMap<String, Box<dyn Tool + Send + Sync>>;

/// Type for a tool factory function
pub type ToolFactory = fn() -> Box<dyn Tool + Send + Sync>;

/// MCP manager builder
pub struct MCPBuilder {
    tools: ToolRegistry,
}

impl MCPBuilder {
    /// Creates a new MCP builder without tools
    pub fn new() -> Self {
        Self {
            tools: ToolRegistry::new(),
        }
    }
    
    /// Registers a tool in the manager
    pub fn with_tool(mut self, name: impl Into<String>, tool: Box<dyn Tool + Send + Sync>) -> Self {
        self.tools.insert(name.into(), tool);
        self
    }
    
    /// Builds an MCP manager with the specified transport
    pub fn build<T: Transport + 'static>(self, transport: T) -> MCP<T> {
        MCP {
            transport,
            tools: self.tools,
        }
    }
}

impl Default for MCPBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP protocol manager
pub struct MCP<T: Transport> {
    transport: T,
    tools: ToolRegistry,
}

impl<T: Transport> MCP<T> {
    /// Starts processing MCP messages
    pub async fn run(&mut self) -> Result<(), TransportError> {
        // Basic implementation to start processing
        // In a complete implementation, here:
        // 1. We would receive JSON messages from the transport
        // 2. We would parse them into Message (via an external adapter)
        // 3. We would execute the requested tools
        // 4. We would send responses

        Ok(())
    }
    
    /// Processes a request and returns a response
    pub async fn process_request(&self, request: Request) -> Response {
        let tool_name = request.tool_name.clone();
        let id = request.id.clone();
        
        // Look for the requested tool
        let result = match self.tools.get(&tool_name) {
            Some(tool) => {
                // Execute the tool
                match tool.execute(request.params).await {
                    Ok(result) => Ok(result),
                    Err(err) => Err(Error::ToolError(err.to_string())),
                }
            },
            None => Err(Error::ToolNotFound(tool_name)),
        };
        
        Response { id, result }
    }
    
    /// Accesses the underlying transport
    pub fn transport(&mut self) -> &mut T {
        &mut self.transport
    }
    
    /// Accesses the tool registry
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock implementation of Transport for testing
    struct MockTransport {
        received_messages: Arc<Mutex<Vec<String>>>,
        messages_to_return: Arc<Mutex<Vec<String>>>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                received_messages: Arc::new(Mutex::new(Vec::new())),
                messages_to_return: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add_message_to_return(&self, message: &str) {
            self.messages_to_return.lock().unwrap().push(message.to_string());
        }

        fn get_received_messages(&self) -> Vec<String> {
            self.received_messages.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn receive(&mut self) -> Result<String, TransportError> {
            let mut messages = self.messages_to_return.lock().unwrap();
            if messages.is_empty() {
                Err(TransportError::ReadError("No messages available".to_string()))
            } else {
                Ok(messages.remove(0))
            }
        }

        async fn send(&mut self, message: &str) -> Result<(), TransportError> {
            self.received_messages.lock().unwrap().push(message.to_string());
            Ok(())
        }
    }

    // Mock implementation of Tool for testing
    struct MockTool {
        name: String,
        calls: Arc<Mutex<Vec<serde_json::Value>>>,
        responses: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                calls: Arc::new(Mutex::new(Vec::new())),
                responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn add_response(&self, param_key: &str, response: serde_json::Value) {
            self.responses.lock().unwrap().insert(param_key.to_string(), response);
        }

        fn get_calls(&self) -> Vec<serde_json::Value> {
            self.calls.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
            self.calls.lock().unwrap().push(params.clone());
            
            // If params has a "key" field, use it to look up the response
            let key = if let Some(k) = params.get("key") {
                k.as_str().unwrap_or_default().to_string()
            } else {
                "default".to_string()
            };

            if let Some(response) = self.responses.lock().unwrap().get(&key) {
                Ok(response.clone())
            } else if key == "error" {
                Err(ToolError::ExecutionError("Test error".to_string()))
            } else {
                Ok(json!({ "result": "default", "tool": self.name }))
            }
        }
    }

    #[tokio::test]
    async fn test_transport_trait() {
        let mut transport = MockTransport::new();
        
        // Test sending a message
        let result = transport.send("test message").await;
        assert!(result.is_ok());
        assert_eq!(transport.get_received_messages(), vec!["test message"]);
        
        // Test receiving a message
        transport.add_message_to_return("response message");
        let received = transport.receive().await;
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), "response message");
        
        // Test error when no messages are available
        let error_result = transport.receive().await;
        assert!(error_result.is_err());
        if let Err(TransportError::ReadError(msg)) = error_result {
            assert_eq!(msg, "No messages available");
        } else {
            panic!("Expected ReadError");
        }
    }

    #[tokio::test]
    async fn test_tool_trait() {
        let tool = MockTool::new("test-tool");
        
        // Add a custom response for a specific key
        tool.add_response("test-key", json!({ "result": "success" }));
        
        // Test execution with default parameters
        let result = tool.execute(json!({ "param": "value" })).await;
        assert!(result.is_ok());
        let result_value = result.unwrap();
        assert_eq!(result_value["result"], "default");
        assert_eq!(result_value["tool"], "test-tool");
        
        // Verify call was recorded
        let calls = tool.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0]["param"], "value");
        
        // Test execution with specific key
        let result = tool.execute(json!({ "key": "test-key" })).await;
        assert!(result.is_ok());
        let result_value = result.unwrap();
        assert_eq!(result_value["result"], "success");
        
        // Test error case
        let error_result = tool.execute(json!({ "key": "error" })).await;
        assert!(error_result.is_err());
        if let Err(ToolError::ExecutionError(msg)) = error_result {
            assert_eq!(msg, "Test error");
        } else {
            panic!("Expected ExecutionError");
        }
    }

    #[tokio::test]
    async fn test_mcp_builder() {
        // Create a builder
        let builder = MCPBuilder::new();
        
        // Add a tool
        let tool = Box::new(MockTool::new("test-tool"));
        let builder = builder.with_tool("test-tool", tool);
        
        // Build MCP with a transport
        let transport = MockTransport::new();
        let mcp = builder.build(transport);
        
        // Verify the tool is registered
        assert!(mcp.tools().contains_key("test-tool"));
        assert_eq!(mcp.tools().len(), 1);
    }

    #[tokio::test]
    async fn test_mcp_integration() {
        // Create a transport
        let transport = MockTransport::new();
        
        // Add a mock tool
        let tool = Box::new(MockTool::new("test-tool"));
        
        // Build the MCP
        let mut mcp = MCPBuilder::new()
            .with_tool("test-tool", tool)
            .build(transport);
        
        // Verify we can access the transport and tools
        assert!(mcp.transport().send("test message").await.is_ok());
        assert!(mcp.tools().contains_key("test-tool"));
        
        // Test the run method
        let result = mcp.run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_transport_error() {
        // Test error variants
        let read_error = TransportError::ReadError("test read error".to_string());
        let write_error = TransportError::WriteError("test write error".to_string());
        let conn_error = TransportError::ConnectionError("test connection error".to_string());
        let json_error = TransportError::JsonError(serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err());
        let other_error = TransportError::Other("generic error".to_string());
        
        // Verify error messages
        assert!(format!("{}", read_error).contains("Read Error: test read error"));
        assert!(format!("{}", write_error).contains("Write Error: test write error"));
        assert!(format!("{}", conn_error).contains("Connection Error: test connection error"));
        assert!(format!("{}", json_error).contains("JSON Error:"));
        assert!(format!("{}", other_error).contains("generic error"));
    }

    #[test]
    fn test_tool_error() {
        // Test error variants
        let invalid_params = ToolError::InvalidParams("test invalid params".to_string());
        let execution_error = ToolError::ExecutionError("test execution error".to_string());
        let tool_not_found = ToolError::ToolNotFound("test tool".to_string());
        let json_error = ToolError::JsonError(serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err());
        let other_error = ToolError::Other("generic error".to_string());
        
        // Verify error messages
        assert!(format!("{}", invalid_params).contains("Invalid Parameters: test invalid params"));
        assert!(format!("{}", execution_error).contains("Execution Error: test execution error"));
        assert!(format!("{}", tool_not_found).contains("Tool Not Found: test tool"));
        assert!(format!("{}", json_error).contains("JSON Error:"));
        assert!(format!("{}", other_error).contains("generic error"));
    }

    #[tokio::test]
    async fn test_domain_messages() {
        // Test Request
        let request = Request {
            id: "req-123".to_string(),
            tool_name: "test-tool".to_string(),
            params: json!({"test": "value"}),
        };
        assert_eq!(request.id, "req-123");
        assert_eq!(request.tool_name, "test-tool");
        assert_eq!(request.params["test"], "value");

        // Test Response with success
        let success_response = Response {
            id: "req-123".to_string(),
            result: Ok(json!({"status": "success"})),
        };
        assert_eq!(success_response.id, "req-123");
        assert!(success_response.result.is_ok());
        
        // Test Response with error
        let error_response = Response {
            id: "req-456".to_string(),
            result: Err(Error::ToolNotFound("missing-tool".to_string())),
        };
        assert_eq!(error_response.id, "req-456");
        assert!(error_response.result.is_err());
        
        // Test Notification
        let notification = Notification {
            event: "tool-completed".to_string(),
            params: json!({"tool": "test-tool", "status": "done"}),
        };
        assert_eq!(notification.event, "tool-completed");
        assert_eq!(notification.params["tool"], "test-tool");
        
        // Test Message enum variants
        let message_request = Message::Request(request);
        let message_response = Message::Response(success_response);
        let message_notification = Message::Notification(notification);
        
        match message_request {
            Message::Request(req) => assert_eq!(req.tool_name, "test-tool"),
            _ => panic!("Expected Request variant"),
        }
        
        match message_response {
            Message::Response(resp) => assert!(resp.result.is_ok()),
            _ => panic!("Expected Response variant"),
        }
        
        match message_notification {
            Message::Notification(notif) => assert_eq!(notif.event, "tool-completed"),
            _ => panic!("Expected Notification variant"),
        }
    }
    
    #[tokio::test]
    async fn test_process_request() {
        // Create a mock tool
        let tool = Box::new(MockTool::new("echo-tool"));
        tool.add_response("echo", json!({"echo": "hello world"}));
        
        // Build the MCP
        let transport = MockTransport::new();
        let mcp = MCPBuilder::new()
            .with_tool("echo-tool", tool)
            .build(transport);
        
        // Test successful request
        let request = Request {
            id: "req-1".to_string(),
            tool_name: "echo-tool".to_string(),
            params: json!({"key": "echo"}),
        };
        
        let response = mcp.process_request(request).await;
        assert_eq!(response.id, "req-1");
        assert!(response.result.is_ok());
        let result = response.result.as_ref().unwrap();
        assert_eq!(result["echo"], "hello world");
        
        // Test request for non-existent tool
        let invalid_request = Request {
            id: "req-2".to_string(),
            tool_name: "nonexistent-tool".to_string(),
            params: json!({}),
        };
        
        let error_response = mcp.process_request(invalid_request).await;
        assert_eq!(error_response.id, "req-2");
        assert!(error_response.result.is_err());
        
        match error_response.result {
            Err(Error::ToolNotFound(name)) => assert_eq!(name, "nonexistent-tool"),
            _ => panic!("Expected ToolNotFound error"),
        }
    }
}