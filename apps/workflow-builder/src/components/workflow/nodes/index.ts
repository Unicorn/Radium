/**
 * Export all custom node types
 */

import { ActivityNode as ActivityNodeComponent } from './ActivityNode';
import { AgentNode as AgentNodeComponent } from './AgentNode';
import { SignalNode as SignalNodeComponent } from './SignalNode';
import { TriggerNode as TriggerNodeComponent } from './TriggerNode';
import { EndNode as EndNodeComponent } from './EndNode';
import { ChildWorkflowNode as ChildWorkflowNodeComponent } from './ChildWorkflowNode';
import { ApiEndpointNode } from './ApiEndpointNode';
import { ConditionNode as ConditionNodeComponent } from './ConditionNode';
import { PhaseNode as PhaseNodeComponent } from './PhaseNode';
import { RetryNode as RetryNodeComponent } from './RetryNode';
import { StateVariableNode as StateVariableNodeComponent } from './StateVariableNode';
import { ServiceContainerNode as ServiceContainerNodeComponent } from './ServiceContainerNode';
import { DataInNode as DataInNodeComponent } from './DataInNode';
import { DataOutNode as DataOutNodeComponent } from './DataOutNode';
import { KongLoggingNode as KongLoggingNodeComponent } from './KongLoggingNode';
import { KongCacheNode as KongCacheNodeComponent } from './KongCacheNode';
import { KongCorsNode as KongCorsNodeComponent } from './KongCorsNode';
import { GraphQLNode as GraphQLNodeComponent } from './GraphQLNode';
import { MCPServerNode as MCPServerNodeComponent } from './MCPServerNode';

export { ActivityNodeComponent as ActivityNode };
export { AgentNodeComponent as AgentNode };
export { SignalNodeComponent as SignalNode };
export { TriggerNodeComponent as TriggerNode };
export { EndNodeComponent as EndNode };
export { ChildWorkflowNodeComponent as ChildWorkflowNode };
export { ApiEndpointNode } from './ApiEndpointNode';
export { ConditionNodeComponent as ConditionNode };
export { PhaseNodeComponent as PhaseNode };
export { RetryNodeComponent as RetryNode };
export { StateVariableNodeComponent as StateVariable };
export { ServiceContainerNodeComponent as ServiceContainerNode };
export { DataInNodeComponent as DataInNode };
export { DataOutNodeComponent as DataOutNode };
export { KongLoggingNodeComponent as KongLoggingNode };
export { KongCacheNodeComponent as KongCacheNode };
export { KongCorsNodeComponent as KongCorsNode };
export { GraphQLNodeComponent as GraphQLNode };
export { MCPServerNodeComponent as MCPServerNode };

// Node type registry for React Flow
export const nodeTypes = {
  activity: ActivityNodeComponent,
  agent: AgentNodeComponent,
  signal: SignalNodeComponent,
  trigger: TriggerNodeComponent,
  end: EndNodeComponent,
  'child-workflow': ChildWorkflowNodeComponent,
  'api-endpoint': ApiEndpointNode,
  condition: ConditionNodeComponent,
  phase: PhaseNodeComponent,
  retry: RetryNodeComponent,
  'state-variable': StateVariableNodeComponent,
  'service-container': ServiceContainerNodeComponent,
  'data-in': DataInNodeComponent,
  'data-out': DataOutNodeComponent,
  'kong-logging': KongLoggingNodeComponent,
  'kong-cache': KongCacheNodeComponent,
  'kong-cors': KongCorsNodeComponent,
  'graphql-gateway': GraphQLNodeComponent,
  'mcp-server': MCPServerNodeComponent,
};

