export type Json =
  | string
  | number
  | boolean
  | null
  | { [key: string]: Json | undefined }
  | Json[]

export type Database = {
  // Allows to automatically instantiate createClient with right options
  // instead of createClient<Database, { PostgrestVersion: 'XX' }>(URL, KEY)
  __InternalSupabase: {
    PostgrestVersion: "13.0.5"
  }
  public: {
    Tables: {
      activities: {
        Row: {
          category: string | null
          created_at: string
          created_by: string
          deprecated: boolean
          deprecated_message: string | null
          deprecated_since: string | null
          description: string | null
          examples: Json | null
          function_name: string
          id: string
          input_schema: Json
          is_active: boolean
          last_used_at: string | null
          migrate_to_activity_id: string | null
          module_path: string
          name: string
          output_schema: Json | null
          package_name: string
          tags: string[] | null
          updated_at: string
          usage_count: number
        }
        Insert: {
          category?: string | null
          created_at?: string
          created_by: string
          deprecated?: boolean
          deprecated_message?: string | null
          deprecated_since?: string | null
          description?: string | null
          examples?: Json | null
          function_name: string
          id?: string
          input_schema: Json
          is_active?: boolean
          last_used_at?: string | null
          migrate_to_activity_id?: string | null
          module_path: string
          name: string
          output_schema?: Json | null
          package_name: string
          tags?: string[] | null
          updated_at?: string
          usage_count?: number
        }
        Update: {
          category?: string | null
          created_at?: string
          created_by?: string
          deprecated?: boolean
          deprecated_message?: string | null
          deprecated_since?: string | null
          description?: string | null
          examples?: Json | null
          function_name?: string
          id?: string
          input_schema?: Json
          is_active?: boolean
          last_used_at?: string | null
          migrate_to_activity_id?: string | null
          module_path?: string
          name?: string
          output_schema?: Json | null
          package_name?: string
          tags?: string[] | null
          updated_at?: string
          usage_count?: number
        }
        Relationships: [
          {
            foreignKeyName: "activities_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "activities_migrate_to_activity_id_fkey"
            columns: ["migrate_to_activity_id"]
            isOneToOne: false
            referencedRelation: "activities"
            referencedColumns: ["id"]
          }
        ]
      }
      activity_categories: {
        Row: {
          created_at: string
          description: string | null
          icon: string | null
          id: string
          name: string
        }
        Insert: {
          created_at?: string
          description?: string | null
          icon?: string | null
          id?: string
          name: string
        }
        Update: {
          created_at?: string
          description?: string | null
          icon?: string | null
          id?: string
          name?: string
        }
        Relationships: []
      }
      agent_prompts: {
        Row: {
          capabilities: string[] | null
          created_at: string
          created_by: string
          deprecated: boolean
          deprecated_message: string | null
          description: string | null
          display_name: string
          id: string
          migrate_to_prompt_id: string | null
          name: string
          prompt_content: string
          prompt_variables: Json | null
          recommended_models: Json | null
          tags: string[] | null
          updated_at: string
          version: string
          visibility_id: string
        }
        Insert: {
          capabilities?: string[] | null
          created_at?: string
          created_by: string
          deprecated?: boolean
          deprecated_message?: string | null
          description?: string | null
          display_name: string
          id?: string
          migrate_to_prompt_id?: string | null
          name: string
          prompt_content: string
          prompt_variables?: Json | null
          recommended_models?: Json | null
          tags?: string[] | null
          updated_at?: string
          version: string
          visibility_id: string
        }
        Update: {
          capabilities?: string[] | null
          created_at?: string
          created_by?: string
          deprecated?: boolean
          deprecated_message?: string | null
          description?: string | null
          display_name?: string
          id?: string
          migrate_to_prompt_id?: string | null
          name?: string
          prompt_content?: string
          prompt_variables?: Json | null
          recommended_models?: Json | null
          tags?: string[] | null
          updated_at?: string
          version?: string
          visibility_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "agent_prompts_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "agent_prompts_migrate_to_prompt_id_fkey"
            columns: ["migrate_to_prompt_id"]
            isOneToOne: false
            referencedRelation: "agent_prompts"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "agent_prompts_visibility_id_fkey"
            columns: ["visibility_id"]
            isOneToOne: false
            referencedRelation: "component_visibility"
            referencedColumns: ["id"]
          },
        ]
      }
      component_types: {
        Row: {
          created_at: string
          description: string | null
          icon: string | null
          id: string
          name: string
        }
        Insert: {
          created_at?: string
          description?: string | null
          icon?: string | null
          id?: string
          name: string
        }
        Update: {
          created_at?: string
          description?: string | null
          icon?: string | null
          id?: string
          name?: string
        }
        Relationships: []
      }
      component_visibility: {
        Row: {
          description: string | null
          id: string
          name: string
        }
        Insert: {
          description?: string | null
          id?: string
          name: string
        }
        Update: {
          description?: string | null
          id?: string
          name?: string
        }
        Relationships: []
      }
      components: {
        Row: {
          agent_prompt_id: string | null
          capabilities: string[] | null
          component_type_id: string
          config_schema: Json | null
          created_at: string
          created_by: string
          deprecated: boolean
          deprecated_message: string | null
          deprecated_since: string | null
          description: string | null
          display_name: string
          id: string
          implementation_path: string | null
          input_schema: Json | null
          migrate_to_component_id: string | null
          model_name: string | null
          model_provider: string | null
          name: string
          npm_package: string | null
          output_schema: Json | null
          tags: string[] | null
          updated_at: string
          version: string
          visibility_id: string
        }
        Insert: {
          agent_prompt_id?: string | null
          capabilities?: string[] | null
          component_type_id: string
          config_schema?: Json | null
          created_at?: string
          created_by: string
          deprecated?: boolean
          deprecated_message?: string | null
          deprecated_since?: string | null
          description?: string | null
          display_name: string
          id?: string
          implementation_path?: string | null
          input_schema?: Json | null
          migrate_to_component_id?: string | null
          model_name?: string | null
          model_provider?: string | null
          name: string
          npm_package?: string | null
          output_schema?: Json | null
          tags?: string[] | null
          updated_at?: string
          version: string
          visibility_id: string
        }
        Update: {
          agent_prompt_id?: string | null
          capabilities?: string[] | null
          component_type_id?: string
          config_schema?: Json | null
          created_at?: string
          created_by?: string
          deprecated?: boolean
          deprecated_message?: string | null
          deprecated_since?: string | null
          description?: string | null
          display_name?: string
          id?: string
          implementation_path?: string | null
          input_schema?: Json | null
          migrate_to_component_id?: string | null
          model_name?: string | null
          model_provider?: string | null
          name?: string
          npm_package?: string | null
          output_schema?: Json | null
          tags?: string[] | null
          updated_at?: string
          version?: string
          visibility_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "components_component_type_id_fkey"
            columns: ["component_type_id"]
            isOneToOne: false
            referencedRelation: "component_types"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "components_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "components_migrate_to_component_id_fkey"
            columns: ["migrate_to_component_id"]
            isOneToOne: false
            referencedRelation: "components"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "components_visibility_id_fkey"
            columns: ["visibility_id"]
            isOneToOne: false
            referencedRelation: "component_visibility"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "fk_components_agent_prompt"
            columns: ["agent_prompt_id"]
            isOneToOne: false
            referencedRelation: "agent_prompts"
            referencedColumns: ["id"]
          },
        ]
      }
      task_queues: {
        Row: {
          created_at: string
          created_by: string
          description: string | null
          id: string
          is_system_queue: boolean
          max_concurrent_activities: number | null
          max_concurrent_workflows: number | null
          name: string
          updated_at: string
        }
        Insert: {
          created_at?: string
          created_by: string
          description?: string | null
          id?: string
          is_system_queue?: boolean
          max_concurrent_activities?: number | null
          max_concurrent_workflows?: number | null
          name: string
          updated_at?: string
        }
        Update: {
          created_at?: string
          created_by?: string
          description?: string | null
          id?: string
          is_system_queue?: boolean
          max_concurrent_activities?: number | null
          max_concurrent_workflows?: number | null
          name?: string
          updated_at?: string
        }
        Relationships: [
          {
            foreignKeyName: "task_queues_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      user_roles: {
        Row: {
          created_at: string
          description: string | null
          id: string
          name: string
          permissions: Json
        }
        Insert: {
          created_at?: string
          description?: string | null
          id?: string
          name: string
          permissions?: Json
        }
        Update: {
          created_at?: string
          description?: string | null
          id?: string
          name?: string
          permissions?: Json
        }
        Relationships: []
      }
      users: {
        Row: {
          auth_user_id: string
          created_at: string
          display_name: string | null
          email: string
          id: string
          last_login_at: string | null
          role_id: string
          updated_at: string
        }
        Insert: {
          auth_user_id: string
          created_at?: string
          display_name?: string | null
          email: string
          id?: string
          last_login_at?: string | null
          role_id: string
          updated_at?: string
        }
        Update: {
          auth_user_id?: string
          created_at?: string
          display_name?: string | null
          email?: string
          id?: string
          last_login_at?: string | null
          role_id?: string
          updated_at?: string
        }
        Relationships: [
          {
            foreignKeyName: "users_role_id_fkey"
            columns: ["role_id"]
            isOneToOne: false
            referencedRelation: "user_roles"
            referencedColumns: ["id"]
          },
        ]
      }
      workflow_edges: {
        Row: {
          config: Json | null
          created_at: string
          edge_id: string
          id: string
          label: string | null
          source_node_id: string
          target_node_id: string
          workflow_id: string
        }
        Insert: {
          config?: Json | null
          created_at?: string
          edge_id: string
          id?: string
          label?: string | null
          source_node_id: string
          target_node_id: string
          workflow_id: string
        }
        Update: {
          config?: Json | null
          created_at?: string
          edge_id?: string
          id?: string
          label?: string | null
          source_node_id?: string
          target_node_id?: string
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_edges_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
      }
      workflow_executions: {
        Row: {
          activities_executed: number | null
          completed_at: string | null
          created_at: string
          duration_ms: number | null
          error_message: string | null
          id: string
          input: Json | null
          output: Json | null
          started_at: string
          status: string
          temporal_run_id: string
          temporal_workflow_id: string
          workflow_id: string
        }
        Insert: {
          activities_executed?: number | null
          completed_at?: string | null
          created_at?: string
          duration_ms?: number | null
          error_message?: string | null
          id?: string
          input?: Json | null
          output?: Json | null
          started_at?: string
          status: string
          temporal_run_id: string
          temporal_workflow_id: string
          workflow_id: string
        }
        Update: {
          activities_executed?: number | null
          completed_at?: string | null
          created_at?: string
          duration_ms?: number | null
          error_message?: string | null
          id?: string
          input?: Json | null
          output?: Json | null
          started_at?: string
          status?: string
          temporal_run_id?: string
          temporal_workflow_id?: string
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_executions_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
      }
      workflow_nodes: {
        Row: {
          block_until_queue: string | null
          block_until_work_items: Json | null
          component_id: string | null
          config: Json
          created_at: string
          id: string
          node_id: string
          node_type: string
          position: Json
          query_parent: string | null
          signal_to_parent: string | null
          work_queue_target: string | null
          workflow_id: string
        }
        Insert: {
          block_until_queue?: string | null
          block_until_work_items?: Json | null
          component_id?: string | null
          config?: Json
          created_at?: string
          id?: string
          node_id: string
          node_type: string
          position: Json
          query_parent?: string | null
          signal_to_parent?: string | null
          work_queue_target?: string | null
          workflow_id: string
        }
        Update: {
          block_until_queue?: string | null
          block_until_work_items?: Json | null
          component_id?: string | null
          config?: Json
          created_at?: string
          id?: string
          node_id?: string
          node_type?: string
          position?: Json
          query_parent?: string | null
          signal_to_parent?: string | null
          work_queue_target?: string | null
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_nodes_component_id_fkey"
            columns: ["component_id"]
            isOneToOne: false
            referencedRelation: "components"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_nodes_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
      }
      workflow_queries: {
        Row: {
          auto_generated: boolean
          created_at: string
          created_by: string
          description: string | null
          id: string
          query_name: string
          return_type: Json | null
          updated_at: string
          work_queue_id: string | null
          workflow_id: string
        }
        Insert: {
          auto_generated?: boolean
          created_at?: string
          created_by: string
          description?: string | null
          id?: string
          query_name: string
          return_type?: Json | null
          updated_at?: string
          work_queue_id?: string | null
          workflow_id: string
        }
        Update: {
          auto_generated?: boolean
          created_at?: string
          created_by?: string
          description?: string | null
          id?: string
          query_name?: string
          return_type?: Json | null
          updated_at?: string
          work_queue_id?: string | null
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_queries_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_queries_work_queue_id_fkey"
            columns: ["work_queue_id"]
            isOneToOne: false
            referencedRelation: "workflow_work_queues"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_queries_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
      }
      workflow_signals: {
        Row: {
          auto_generated: boolean
          created_at: string
          created_by: string
          description: string | null
          id: string
          parameters: Json | null
          signal_name: string
          updated_at: string
          work_queue_id: string | null
          workflow_id: string
        }
        Insert: {
          auto_generated?: boolean
          created_at?: string
          created_by: string
          description?: string | null
          id?: string
          parameters?: Json | null
          signal_name: string
          updated_at?: string
          work_queue_id?: string | null
          workflow_id: string
        }
        Update: {
          auto_generated?: boolean
          created_at?: string
          created_by?: string
          description?: string | null
          id?: string
          parameters?: Json | null
          signal_name?: string
          updated_at?: string
          work_queue_id?: string | null
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_signals_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_signals_work_queue_id_fkey"
            columns: ["work_queue_id"]
            isOneToOne: false
            referencedRelation: "workflow_work_queues"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_signals_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
      }
      workflow_statuses: {
        Row: {
          color: string | null
          description: string | null
          id: string
          name: string
        }
        Insert: {
          color?: string | null
          description?: string | null
          id?: string
          name: string
        }
        Update: {
          color?: string | null
          description?: string | null
          id?: string
          name?: string
        }
        Relationships: []
      }
      workflow_work_queues: {
        Row: {
          created_at: string
          created_by: string
          deduplicate: boolean
          description: string | null
          id: string
          max_size: number | null
          priority: string
          query_name: string
          queue_name: string
          signal_name: string
          updated_at: string
          work_item_schema: Json | null
          workflow_id: string
        }
        Insert: {
          created_at?: string
          created_by: string
          deduplicate?: boolean
          description?: string | null
          id?: string
          max_size?: number | null
          priority?: string
          query_name: string
          queue_name: string
          signal_name: string
          updated_at?: string
          work_item_schema?: Json | null
          workflow_id: string
        }
        Update: {
          created_at?: string
          created_by?: string
          deduplicate?: boolean
          description?: string | null
          id?: string
          max_size?: number | null
          priority?: string
          query_name?: string
          queue_name?: string
          signal_name?: string
          updated_at?: string
          work_item_schema?: Json | null
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_work_queues_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_work_queues_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
      }
      workflows: {
        Row: {
          compiled_typescript: string | null
          created_at: string
          created_by: string
          definition: Json
          deployed_at: string | null
          description: string | null
          display_name: string
          end_with_parent: boolean | null
          execution_timeout_seconds: number | null
          id: string
          is_scheduled: boolean
          last_run_at: string | null
          max_concurrent_executions: number | null
          max_runs: number | null
          name: string
          next_run_at: string | null
          parent_workflow_id: string | null
          query_parent_name: string | null
          run_count: number | null
          schedule_spec: string | null
          signal_to_parent_name: string | null
          start_immediately: boolean | null
          status_id: string
          task_queue_id: string
          temporal_workflow_id: string | null
          temporal_workflow_type: string | null
          updated_at: string
          version: string
          visibility_id: string
        }
        Insert: {
          compiled_typescript?: string | null
          created_at?: string
          created_by: string
          definition: Json
          deployed_at?: string | null
          description?: string | null
          display_name: string
          end_with_parent?: boolean | null
          execution_timeout_seconds?: number | null
          id?: string
          is_scheduled?: boolean
          last_run_at?: string | null
          max_concurrent_executions?: number | null
          max_runs?: number | null
          name: string
          next_run_at?: string | null
          parent_workflow_id?: string | null
          query_parent_name?: string | null
          run_count?: number | null
          schedule_spec?: string | null
          signal_to_parent_name?: string | null
          start_immediately?: boolean | null
          status_id: string
          task_queue_id: string
          temporal_workflow_id?: string | null
          temporal_workflow_type?: string | null
          updated_at?: string
          version?: string
          visibility_id: string
        }
        Update: {
          compiled_typescript?: string | null
          created_at?: string
          created_by?: string
          definition?: Json
          deployed_at?: string | null
          description?: string | null
          display_name?: string
          end_with_parent?: boolean | null
          execution_timeout_seconds?: number | null
          id?: string
          is_scheduled?: boolean
          last_run_at?: string | null
          max_concurrent_executions?: number | null
          max_runs?: number | null
          name?: string
          next_run_at?: string | null
          parent_workflow_id?: string | null
          query_parent_name?: string | null
          run_count?: number | null
          schedule_spec?: string | null
          signal_to_parent_name?: string | null
          start_immediately?: boolean | null
          status_id?: string
          task_queue_id?: string
          temporal_workflow_id?: string | null
          temporal_workflow_type?: string | null
          updated_at?: string
          version?: string
          visibility_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflows_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflows_parent_workflow_id_fkey"
            columns: ["parent_workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflows_status_id_fkey"
            columns: ["status_id"]
            isOneToOne: false
            referencedRelation: "workflow_statuses"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflows_task_queue_id_fkey"
            columns: ["task_queue_id"]
            isOneToOne: false
            referencedRelation: "task_queues"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflows_visibility_id_fkey"
            columns: ["visibility_id"]
            isOneToOne: false
            referencedRelation: "component_visibility"
            referencedColumns: ["id"]
          },
        ]
      }
    }
    Views: {
      [_ in never]: never
    }
    Functions: {
      check_circular_block_dependencies: {
        Args: {
          p_block_queue: string
          p_node_id: string
          p_workflow_id: string
        }
        Returns: boolean
      }
      ensure_default_task_queue: { Args: never; Returns: undefined }
      validate_cron_expression: {
        Args: { cron_expr: string }
        Returns: boolean
      }
    }
    Enums: {
      [_ in never]: never
    }
    CompositeTypes: {
      [_ in never]: never
    }
  }
}

type DatabaseWithoutInternals = Omit<Database, "__InternalSupabase">

type DefaultSchema = DatabaseWithoutInternals[Extract<keyof Database, "public">]

export type Tables<
  DefaultSchemaTableNameOrOptions extends
    | keyof (DefaultSchema["Tables"] & DefaultSchema["Views"])
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof (DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"] &
        DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Views"])
    : never = never,
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? (DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"] &
      DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Views"])[TableName] extends {
      Row: infer R
    }
    ? R
    : never
  : DefaultSchemaTableNameOrOptions extends keyof (DefaultSchema["Tables"] &
        DefaultSchema["Views"])
    ? (DefaultSchema["Tables"] &
        DefaultSchema["Views"])[DefaultSchemaTableNameOrOptions] extends {
        Row: infer R
      }
      ? R
      : never
    : never

export type TablesInsert<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Insert: infer I
    }
    ? I
    : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
    ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
        Insert: infer I
      }
      ? I
      : never
    : never

export type TablesUpdate<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Update: infer U
    }
    ? U
    : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
    ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
        Update: infer U
      }
      ? U
      : never
    : never

export type Enums<
  DefaultSchemaEnumNameOrOptions extends
    | keyof DefaultSchema["Enums"]
    | { schema: keyof DatabaseWithoutInternals },
  EnumName extends DefaultSchemaEnumNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"]
    : never = never,
> = DefaultSchemaEnumNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? DatabaseWithoutInternals[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"][EnumName]
  : DefaultSchemaEnumNameOrOptions extends keyof DefaultSchema["Enums"]
    ? DefaultSchema["Enums"][DefaultSchemaEnumNameOrOptions]
    : never

export type CompositeTypes<
  PublicCompositeTypeNameOrOptions extends
    | keyof DefaultSchema["CompositeTypes"]
    | { schema: keyof DatabaseWithoutInternals },
  CompositeTypeName extends PublicCompositeTypeNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof DatabaseWithoutInternals[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"]
    : never = never,
> = PublicCompositeTypeNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? DatabaseWithoutInternals[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"][CompositeTypeName]
  : PublicCompositeTypeNameOrOptions extends keyof DefaultSchema["CompositeTypes"]
    ? DefaultSchema["CompositeTypes"][PublicCompositeTypeNameOrOptions]
    : never

export const Constants = {
  public: {
    Enums: {},
  },
} as const
