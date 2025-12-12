/**
 * Interface Component Manager
 * Manages creation and linking of service interfaces from interface components
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { toKebabCase } from '@/lib/compiler/utils/ast-helpers';

export interface InterfaceComponentConfig {
  endpointPath: string;
  httpMethod: 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE';
  inputSchema?: Record<string, any>;
  outputSchema?: Record<string, any>;
  isPublic?: boolean;
}

/**
 * Create a service interface from an interface component
 */
export async function createServiceInterfaceFromComponent(
  componentId: string,
  workflowId: string,
  config: InterfaceComponentConfig,
  supabase: SupabaseClient<Database>
): Promise<Database['public']['Tables']['service_interfaces']['Row']> {
  // Get component details with component type
  const { data: component, error: componentError } = await supabase
    .from('components')
    .select(`
      *,
      component_type:component_types(id, name)
    `)
    .eq('id', componentId)
    .single();

  if (componentError || !component) {
    throw new Error(`Component not found: ${componentId}`);
  }

  // Determine interface type based on component type
  const componentType = (component as any).component_type?.name;
  let interfaceType: 'signal' | 'query' | 'update' | 'start_child';
  
  if (componentType === 'data-in') {
    interfaceType = 'signal';
  } else if (componentType === 'data-out') {
    interfaceType = 'query';
  } else {
    throw new Error(`Component type ${componentType} is not an interface component`);
  }

  // Generate callable name from component name
  const callableName = toKebabCase(component.name);

  // Create service interface
  const { data: serviceInterface, error: siError } = await supabase
    .from('service_interfaces')
    .insert({
      workflow_id: workflowId,
      name: component.name,
      display_name: component.display_name,
      description: component.description || `Interface for ${component.display_name}`,
      interface_type: interfaceType,
      callable_name: callableName,
      input_schema: config.inputSchema || component.input_schema || null,
      output_schema: config.outputSchema || component.output_schema || null,
      activity_connection_id: componentId, // Link to component
      is_public: config.isPublic ?? false,
    })
    .select()
    .single();

  if (siError) {
    if (siError.code === '23505') {
      // Unique constraint violation - interface already exists
      // Try to get existing interface
      const { data: existing } = await supabase
        .from('service_interfaces')
        .select('*')
        .eq('workflow_id', workflowId)
        .eq('name', component.name)
        .single();

      if (existing) {
        // Update existing interface
        const { data: updated, error: updateError } = await supabase
          .from('service_interfaces')
          .update({
            display_name: component.display_name,
            description: component.description || `Interface for ${component.display_name}`,
            interface_type: interfaceType,
            callable_name: callableName,
            input_schema: config.inputSchema || component.input_schema || null,
            output_schema: config.outputSchema || component.output_schema || null,
            activity_connection_id: componentId,
            is_public: config.isPublic ?? false,
            updated_at: new Date().toISOString(),
          })
          .eq('id', existing.id)
          .select()
          .single();

        if (updateError) {
          throw new Error(`Failed to update service interface: ${updateError.message}`);
        }

        return updated;
      }
    }
    throw new Error(`Failed to create service interface: ${siError.message}`);
  }

  return serviceInterface;
}

/**
 * Get service interface for a component
 */
export async function getServiceInterfaceForComponent(
  componentId: string,
  workflowId: string,
  supabase: SupabaseClient<Database>
): Promise<Database['public']['Tables']['service_interfaces']['Row'] | null> {
  const { data, error } = await supabase
    .from('service_interfaces')
    .select('*')
    .eq('workflow_id', workflowId)
    .eq('activity_connection_id', componentId)
    .single();

  if (error) {
    if (error.code === 'PGRST116') {
      // No rows returned
      return null;
    }
    throw new Error(`Failed to get service interface: ${error.message}`);
  }

  return data;
}

/**
 * Delete service interface for a component
 */
export async function deleteServiceInterfaceForComponent(
  componentId: string,
  workflowId: string,
  supabase: SupabaseClient<Database>
): Promise<void> {
  const { error } = await supabase
    .from('service_interfaces')
    .delete()
    .eq('workflow_id', workflowId)
    .eq('activity_connection_id', componentId);

  if (error) {
    throw new Error(`Failed to delete service interface: ${error.message}`);
  }
}

