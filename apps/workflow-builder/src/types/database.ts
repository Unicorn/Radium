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
          },
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
      activity_statistics: {
        Row: {
          activity_name: string
          avg_duration_ms: number | null
          created_at: string
          execution_count: number | null
          failure_count: number | null
          id: string
          last_executed_at: string | null
          p95_duration_ms: number | null
          p99_duration_ms: number | null
          project_id: string
          requires_dedicated_worker: boolean | null
          success_count: number | null
          updated_at: string
        }
        Insert: {
          activity_name: string
          avg_duration_ms?: number | null
          created_at?: string
          execution_count?: number | null
          failure_count?: number | null
          id?: string
          last_executed_at?: string | null
          p95_duration_ms?: number | null
          p99_duration_ms?: number | null
          project_id: string
          requires_dedicated_worker?: boolean | null
          success_count?: number | null
          updated_at?: string
        }
        Update: {
          activity_name?: string
          avg_duration_ms?: number | null
          created_at?: string
          execution_count?: number | null
          failure_count?: number | null
          id?: string
          last_executed_at?: string | null
          p95_duration_ms?: number | null
          p99_duration_ms?: number | null
          project_id?: string
          requires_dedicated_worker?: boolean | null
          success_count?: number | null
          updated_at?: string
        }
        Relationships: [
          {
            foreignKeyName: "activity_statistics_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
        ]
      }
      agent_builder_sessions: {
        Row: {
          completed_at: string | null
          conversation_messages: Json | null
          created_at: string
          id: string
          message_count: number | null
          resulting_prompt_id: string | null
          started_at: string
          status: string
          updated_at: string
          user_id: string
        }
        Insert: {
          completed_at?: string | null
          conversation_messages?: Json | null
          created_at?: string
          id?: string
          message_count?: number | null
          resulting_prompt_id?: string | null
          started_at?: string
          status?: string
          updated_at?: string
          user_id: string
        }
        Update: {
          completed_at?: string | null
          conversation_messages?: Json | null
          created_at?: string
          id?: string
          message_count?: number | null
          resulting_prompt_id?: string | null
          started_at?: string
          status?: string
          updated_at?: string
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "agent_builder_sessions_resulting_prompt_id_fkey"
            columns: ["resulting_prompt_id"]
            isOneToOne: false
            referencedRelation: "agent_prompts"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "agent_builder_sessions_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
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
      agent_test_sessions: {
        Row: {
          agent_prompt_id: string
          completed_at: string | null
          conversation_history: Json | null
          created_at: string
          id: string
          message_count: number | null
          started_at: string
          status: string
          temporal_run_id: string
          temporal_workflow_id: string
          updated_at: string
          user_id: string
        }
        Insert: {
          agent_prompt_id: string
          completed_at?: string | null
          conversation_history?: Json | null
          created_at?: string
          id?: string
          message_count?: number | null
          started_at?: string
          status?: string
          temporal_run_id: string
          temporal_workflow_id: string
          updated_at?: string
          user_id: string
        }
        Update: {
          agent_prompt_id?: string
          completed_at?: string | null
          conversation_history?: Json | null
          created_at?: string
          id?: string
          message_count?: number | null
          started_at?: string
          status?: string
          temporal_run_id?: string
          temporal_workflow_id?: string
          updated_at?: string
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "agent_test_sessions_agent_prompt_id_fkey"
            columns: ["agent_prompt_id"]
            isOneToOne: false
            referencedRelation: "agent_prompts"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "agent_test_sessions_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      component_metrics: {
        Row: {
          attempt_number: number | null
          completed_at: string | null
          component_id: string | null
          component_name: string
          component_type: string
          cpu_time_ms: number | null
          duration_ms: number | null
          error_code: string | null
          error_type: string | null
          id: string
          invocation_count: number
          io_bytes: number | null
          is_retry: boolean | null
          memory_peak_mb: number | null
          metadata: Json | null
          node_id: string | null
          project_id: string
          queue_time_ms: number | null
          recorded_at: string | null
          started_at: string
          status: string
          workflow_execution_id: string | null
          workflow_id: string
        }
        Insert: {
          attempt_number?: number | null
          completed_at?: string | null
          component_id?: string | null
          component_name: string
          component_type: string
          cpu_time_ms?: number | null
          duration_ms?: number | null
          error_code?: string | null
          error_type?: string | null
          id?: string
          invocation_count?: number
          io_bytes?: number | null
          is_retry?: boolean | null
          memory_peak_mb?: number | null
          metadata?: Json | null
          node_id?: string | null
          project_id: string
          queue_time_ms?: number | null
          recorded_at?: string | null
          started_at: string
          status: string
          workflow_execution_id?: string | null
          workflow_id: string
        }
        Update: {
          attempt_number?: number | null
          completed_at?: string | null
          component_id?: string | null
          component_name?: string
          component_type?: string
          cpu_time_ms?: number | null
          duration_ms?: number | null
          error_code?: string | null
          error_type?: string | null
          id?: string
          invocation_count?: number
          io_bytes?: number | null
          is_retry?: boolean | null
          memory_peak_mb?: number | null
          metadata?: Json | null
          node_id?: string | null
          project_id?: string
          queue_time_ms?: number | null
          recorded_at?: string | null
          started_at?: string
          status?: string
          workflow_execution_id?: string | null
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "component_metrics_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "component_metrics_workflow_execution_id_fkey"
            columns: ["workflow_execution_id"]
            isOneToOne: false
            referencedRelation: "workflow_executions"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "component_metrics_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
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
      component_usage_daily: {
        Row: {
          avg_duration_ms: number | null
          component_name: string
          component_type: string
          date: string
          failed_invocations: number
          id: string
          max_duration_ms: number | null
          p50_duration_ms: number | null
          p95_duration_ms: number | null
          p99_duration_ms: number | null
          project_id: string
          retried_invocations: number
          successful_invocations: number
          total_cpu_time_ms: number | null
          total_duration_ms: number | null
          total_invocations: number
          total_memory_mb: number | null
          updated_at: string | null
        }
        Insert: {
          avg_duration_ms?: number | null
          component_name: string
          component_type: string
          date: string
          failed_invocations?: number
          id?: string
          max_duration_ms?: number | null
          p50_duration_ms?: number | null
          p95_duration_ms?: number | null
          p99_duration_ms?: number | null
          project_id: string
          retried_invocations?: number
          successful_invocations?: number
          total_cpu_time_ms?: number | null
          total_duration_ms?: number | null
          total_invocations?: number
          total_memory_mb?: number | null
          updated_at?: string | null
        }
        Update: {
          avg_duration_ms?: number | null
          component_name?: string
          component_type?: string
          date?: string
          failed_invocations?: number
          id?: string
          max_duration_ms?: number | null
          p50_duration_ms?: number | null
          p95_duration_ms?: number | null
          p99_duration_ms?: number | null
          project_id?: string
          retried_invocations?: number
          successful_invocations?: number
          total_cpu_time_ms?: number | null
          total_duration_ms?: number | null
          total_invocations?: number
          total_memory_mb?: number | null
          updated_at?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "component_usage_daily_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
        ]
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
      connector_classifications: {
        Row: {
          classification: string
          connector_id: string
          created_at: string | null
          id: string
        }
        Insert: {
          classification: string
          connector_id: string
          created_at?: string | null
          id?: string
        }
        Update: {
          classification?: string
          connector_id?: string
          created_at?: string | null
          id?: string
        }
        Relationships: [
          {
            foreignKeyName: "connector_classifications_connector_id_fkey"
            columns: ["connector_id"]
            isOneToOne: false
            referencedRelation: "connectors"
            referencedColumns: ["id"]
          },
        ]
      }
      connectors: {
        Row: {
          classifications: Json | null
          config_data: Json
          config_schema: Json
          connector_type: string
          created_at: string
          created_by: string
          credentials_encrypted: string | null
          description: string | null
          display_name: string
          id: string
          is_active: boolean | null
          name: string
          oauth_config: Json | null
          project_id: string
          updated_at: string
        }
        Insert: {
          classifications?: Json | null
          config_data: Json
          config_schema: Json
          connector_type: string
          created_at?: string
          created_by: string
          credentials_encrypted?: string | null
          description?: string | null
          display_name: string
          id?: string
          is_active?: boolean | null
          name: string
          oauth_config?: Json | null
          project_id: string
          updated_at?: string
        }
        Update: {
          classifications?: Json | null
          config_data?: Json
          config_schema?: Json
          connector_type?: string
          created_at?: string
          created_by?: string
          credentials_encrypted?: string | null
          description?: string | null
          display_name?: string
          id?: string
          is_active?: boolean | null
          name?: string
          oauth_config?: Json | null
          project_id?: string
          updated_at?: string
        }
        Relationships: [
          {
            foreignKeyName: "connectors_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "connectors_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
        ]
      }
      project_connectors: {
        Row: {
          auth_config: Json | null
          created_at: string
          created_by: string
          description: string | null
          display_name: string
          id: string
          name: string
          nexus_endpoint_name: string
          source_project_id: string
          target_interface_id: string | null
          target_project_id: string
          target_service_id: string
          updated_at: string
          visibility: string | null
        }
        Insert: {
          auth_config?: Json | null
          created_at?: string
          created_by: string
          description?: string | null
          display_name: string
          id?: string
          name: string
          nexus_endpoint_name: string
          source_project_id: string
          target_interface_id?: string | null
          target_project_id: string
          target_service_id: string
          updated_at?: string
          visibility?: string | null
        }
        Update: {
          auth_config?: Json | null
          created_at?: string
          created_by?: string
          description?: string | null
          display_name?: string
          id?: string
          name?: string
          nexus_endpoint_name?: string
          source_project_id?: string
          target_interface_id?: string | null
          target_project_id?: string
          target_service_id?: string
          updated_at?: string
          visibility?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "project_connectors_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "project_connectors_source_project_id_fkey"
            columns: ["source_project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "project_connectors_target_project_id_fkey"
            columns: ["target_project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "project_connectors_target_service_id_fkey"
            columns: ["target_service_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
      }
      project_state_variables: {
        Row: {
          created_at: string | null
          id: string
          name: string
          project_id: string
          schema: Json | null
          storage_config: Json | null
          storage_type: string
          type: string
          updated_at: string | null
        }
        Insert: {
          created_at?: string | null
          id?: string
          name: string
          project_id: string
          schema?: Json | null
          storage_config?: Json | null
          storage_type: string
          type: string
          updated_at?: string | null
        }
        Update: {
          created_at?: string | null
          id?: string
          name?: string
          project_id?: string
          schema?: Json | null
          storage_config?: Json | null
          storage_type?: string
          type?: string
          updated_at?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "project_state_variables_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
        ]
      }
      projects: {
        Row: {
          avg_execution_duration_ms: number | null
          created_at: string
          created_by: string
          description: string | null
          id: string
          is_active: boolean
          is_archived: boolean
          is_default: boolean
          last_execution_at: string | null
          name: string
          task_queue_name: string
          total_activity_executions: number | null
          total_workflow_executions: number | null
          updated_at: string
        }
        Insert: {
          avg_execution_duration_ms?: number | null
          created_at?: string
          created_by: string
          description?: string | null
          id?: string
          is_active?: boolean
          is_archived?: boolean
          is_default?: boolean
          last_execution_at?: string | null
          name: string
          task_queue_name: string
          total_activity_executions?: number | null
          total_workflow_executions?: number | null
          updated_at?: string
        }
        Update: {
          avg_execution_duration_ms?: number | null
          created_at?: string
          created_by?: string
          description?: string | null
          id?: string
          is_active?: boolean
          is_archived?: boolean
          is_default?: boolean
          last_execution_at?: string | null
          name?: string
          task_queue_name?: string
          total_activity_executions?: number | null
          total_workflow_executions?: number | null
          updated_at?: string
        }
        Relationships: [
          {
            foreignKeyName: "projects_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      resource_events: {
        Row: {
          completed_at: string | null
          completion_tokens: number | null
          component_metric_id: string | null
          direction: string | null
          duration_ms: number | null
          error_code: string | null
          error_type: string | null
          id: string
          latency_ms: number | null
          metadata: Json | null
          model_name: string | null
          operation: string
          project_id: string
          prompt_tokens: number | null
          recorded_at: string | null
          request_size_bytes: number | null
          resource_id: string | null
          resource_name: string
          resource_subtype: string | null
          resource_type: string
          response_size_bytes: number | null
          started_at: string
          status: string
          target_project_id: string | null
          target_service: string | null
          total_tokens: number | null
          workflow_execution_id: string | null
          workflow_id: string | null
        }
        Insert: {
          completed_at?: string | null
          completion_tokens?: number | null
          component_metric_id?: string | null
          direction?: string | null
          duration_ms?: number | null
          error_code?: string | null
          error_type?: string | null
          id?: string
          latency_ms?: number | null
          metadata?: Json | null
          model_name?: string | null
          operation: string
          project_id: string
          prompt_tokens?: number | null
          recorded_at?: string | null
          request_size_bytes?: number | null
          resource_id?: string | null
          resource_name: string
          resource_subtype?: string | null
          resource_type: string
          response_size_bytes?: number | null
          started_at: string
          status?: string
          target_project_id?: string | null
          target_service?: string | null
          total_tokens?: number | null
          workflow_execution_id?: string | null
          workflow_id?: string | null
        }
        Update: {
          completed_at?: string | null
          completion_tokens?: number | null
          component_metric_id?: string | null
          direction?: string | null
          duration_ms?: number | null
          error_code?: string | null
          error_type?: string | null
          id?: string
          latency_ms?: number | null
          metadata?: Json | null
          model_name?: string | null
          operation?: string
          project_id?: string
          prompt_tokens?: number | null
          recorded_at?: string | null
          request_size_bytes?: number | null
          resource_id?: string | null
          resource_name?: string
          resource_subtype?: string | null
          resource_type?: string
          response_size_bytes?: number | null
          started_at?: string
          status?: string
          target_project_id?: string | null
          target_service?: string | null
          total_tokens?: number | null
          workflow_execution_id?: string | null
          workflow_id?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "resource_events_component_metric_id_fkey"
            columns: ["component_metric_id"]
            isOneToOne: false
            referencedRelation: "component_metrics"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "resource_events_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "resource_events_workflow_execution_id_fkey"
            columns: ["workflow_execution_id"]
            isOneToOne: false
            referencedRelation: "workflow_executions"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "resource_events_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
      }
      resource_usage_daily: {
        Row: {
          avg_duration_ms: number | null
          avg_latency_ms: number | null
          date: string
          failed_invocations: number
          id: string
          project_id: string
          resource_name: string
          resource_type: string
          successful_invocations: number
          total_completion_tokens: number | null
          total_duration_ms: number | null
          total_invocations: number
          total_prompt_tokens: number | null
          total_request_bytes: number | null
          total_response_bytes: number | null
          total_tokens: number | null
          updated_at: string | null
        }
        Insert: {
          avg_duration_ms?: number | null
          avg_latency_ms?: number | null
          date: string
          failed_invocations?: number
          id?: string
          project_id: string
          resource_name: string
          resource_type: string
          successful_invocations?: number
          total_completion_tokens?: number | null
          total_duration_ms?: number | null
          total_invocations?: number
          total_prompt_tokens?: number | null
          total_request_bytes?: number | null
          total_response_bytes?: number | null
          total_tokens?: number | null
          updated_at?: string | null
        }
        Update: {
          avg_duration_ms?: number | null
          avg_latency_ms?: number | null
          date?: string
          failed_invocations?: number
          id?: string
          project_id?: string
          resource_name?: string
          resource_type?: string
          successful_invocations?: number
          total_completion_tokens?: number | null
          total_duration_ms?: number | null
          total_invocations?: number
          total_prompt_tokens?: number | null
          total_request_bytes?: number | null
          total_response_bytes?: number | null
          total_tokens?: number | null
          updated_at?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "resource_usage_daily_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
        ]
      }
      state_variable_metrics: {
        Row: {
          access_count: number | null
          created_at: string | null
          id: string
          last_accessed: string | null
          scope: string
          size_bytes: number | null
          variable_id: string
        }
        Insert: {
          access_count?: number | null
          created_at?: string | null
          id?: string
          last_accessed?: string | null
          scope: string
          size_bytes?: number | null
          variable_id: string
        }
        Update: {
          access_count?: number | null
          created_at?: string | null
          id?: string
          last_accessed?: string | null
          scope?: string
          size_bytes?: number | null
          variable_id?: string
        }
        Relationships: []
      }
      task_queues: {
        Row: {
          created_at: string
          created_by: string
          description: string | null
          display_name: string
          id: string
          is_default: boolean
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
          display_name: string
          id?: string
          is_default?: boolean
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
          display_name?: string
          id?: string
          is_default?: boolean
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
      workflow_compiled_code: {
        Row: {
          activities_code: string
          avg_execution_duration_ms: number | null
          compiled_at: string
          compiled_by: string | null
          error_count: number | null
          execution_count: number | null
          id: string
          is_active: boolean
          last_executed_at: string | null
          package_json: string
          tsconfig_json: string
          version: string
          worker_code: string
          workflow_code: string
          workflow_id: string
        }
        Insert: {
          activities_code: string
          avg_execution_duration_ms?: number | null
          compiled_at?: string
          compiled_by?: string | null
          error_count?: number | null
          execution_count?: number | null
          id?: string
          is_active?: boolean
          last_executed_at?: string | null
          package_json: string
          tsconfig_json: string
          version: string
          worker_code: string
          workflow_code: string
          workflow_id: string
        }
        Update: {
          activities_code?: string
          avg_execution_duration_ms?: number | null
          compiled_at?: string
          compiled_by?: string | null
          error_count?: number | null
          execution_count?: number | null
          id?: string
          is_active?: boolean
          last_executed_at?: string | null
          package_json?: string
          tsconfig_json?: string
          version?: string
          worker_code?: string
          workflow_code?: string
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_compiled_code_compiled_by_fkey"
            columns: ["compiled_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_compiled_code_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
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
      workflow_execution_metrics: {
        Row: {
          activity_count: number | null
          completed_at: string | null
          duration_ms: number | null
          error_message: string | null
          error_type: string | null
          id: string
          input_size_bytes: number | null
          metadata: Json | null
          output_size_bytes: number | null
          project_id: string
          queue_time_ms: number | null
          recorded_at: string | null
          retry_count: number | null
          started_at: string
          status: string
          task_queue_name: string | null
          temporal_run_id: string | null
          temporal_workflow_id: string | null
          total_cpu_time_ms: number | null
          total_memory_mb: number | null
          trigger_source: string | null
          trigger_type: string
          workflow_execution_id: string | null
          workflow_id: string
          workflow_name: string
          workflow_version: string | null
        }
        Insert: {
          activity_count?: number | null
          completed_at?: string | null
          duration_ms?: number | null
          error_message?: string | null
          error_type?: string | null
          id?: string
          input_size_bytes?: number | null
          metadata?: Json | null
          output_size_bytes?: number | null
          project_id: string
          queue_time_ms?: number | null
          recorded_at?: string | null
          retry_count?: number | null
          started_at?: string
          status?: string
          task_queue_name?: string | null
          temporal_run_id?: string | null
          temporal_workflow_id?: string | null
          total_cpu_time_ms?: number | null
          total_memory_mb?: number | null
          trigger_source?: string | null
          trigger_type?: string
          workflow_execution_id?: string | null
          workflow_id: string
          workflow_name: string
          workflow_version?: string | null
        }
        Update: {
          activity_count?: number | null
          completed_at?: string | null
          duration_ms?: number | null
          error_message?: string | null
          error_type?: string | null
          id?: string
          input_size_bytes?: number | null
          metadata?: Json | null
          output_size_bytes?: number | null
          project_id?: string
          queue_time_ms?: number | null
          recorded_at?: string | null
          retry_count?: number | null
          started_at?: string
          status?: string
          task_queue_name?: string | null
          temporal_run_id?: string | null
          temporal_workflow_id?: string | null
          total_cpu_time_ms?: number | null
          total_memory_mb?: number | null
          trigger_source?: string | null
          trigger_type?: string
          workflow_execution_id?: string | null
          workflow_id?: string
          workflow_name?: string
          workflow_version?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "workflow_execution_metrics_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_execution_metrics_workflow_execution_id_fkey"
            columns: ["workflow_execution_id"]
            isOneToOne: false
            referencedRelation: "workflow_executions"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_execution_metrics_workflow_id_fkey"
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
          created_by: string | null
          duration_ms: number | null
          error_message: string | null
          id: string
          input: Json | null
          output: Json | null
          started_at: string
          status: string
          temporal_run_id: string | null
          temporal_workflow_id: string | null
          updated_at: string
          workflow_id: string
        }
        Insert: {
          activities_executed?: number | null
          completed_at?: string | null
          created_at?: string
          created_by?: string | null
          duration_ms?: number | null
          error_message?: string | null
          id?: string
          input?: Json | null
          output?: Json | null
          started_at?: string
          status: string
          temporal_run_id?: string | null
          temporal_workflow_id?: string | null
          updated_at?: string
          workflow_id: string
        }
        Update: {
          activities_executed?: number | null
          completed_at?: string | null
          created_at?: string
          created_by?: string | null
          duration_ms?: number | null
          error_message?: string | null
          id?: string
          input?: Json | null
          output?: Json | null
          started_at?: string
          status?: string
          temporal_run_id?: string | null
          temporal_workflow_id?: string | null
          updated_at?: string
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_executions_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
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
      workflow_state_variables: {
        Row: {
          created_at: string | null
          id: string
          name: string
          schema: Json | null
          storage_config: Json | null
          storage_type: string
          type: string
          updated_at: string | null
          workflow_id: string
        }
        Insert: {
          created_at?: string | null
          id?: string
          name: string
          schema?: Json | null
          storage_config?: Json | null
          storage_type: string
          type: string
          updated_at?: string | null
          workflow_id: string
        }
        Update: {
          created_at?: string | null
          id?: string
          name?: string
          schema?: Json | null
          storage_config?: Json | null
          storage_type?: string
          type?: string
          updated_at?: string | null
          workflow_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_state_variables_workflow_id_fkey"
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
      workflow_usage_daily: {
        Row: {
          api_triggers: number | null
          avg_activities_per_execution: number | null
          avg_duration_ms: number | null
          cancelled_executions: number
          date: string
          failed_executions: number
          id: string
          manual_triggers: number | null
          max_duration_ms: number | null
          p50_duration_ms: number | null
          p95_duration_ms: number | null
          p99_duration_ms: number | null
          project_id: string
          schedule_triggers: number | null
          successful_executions: number
          timeout_executions: number
          total_activities_executed: number | null
          total_duration_ms: number | null
          total_executions: number
          total_input_bytes: number | null
          total_output_bytes: number | null
          updated_at: string | null
          webhook_triggers: number | null
          workflow_id: string
          workflow_name: string
        }
        Insert: {
          api_triggers?: number | null
          avg_activities_per_execution?: number | null
          avg_duration_ms?: number | null
          cancelled_executions?: number
          date: string
          failed_executions?: number
          id?: string
          manual_triggers?: number | null
          max_duration_ms?: number | null
          p50_duration_ms?: number | null
          p95_duration_ms?: number | null
          p99_duration_ms?: number | null
          project_id: string
          schedule_triggers?: number | null
          successful_executions?: number
          timeout_executions?: number
          total_activities_executed?: number | null
          total_duration_ms?: number | null
          total_executions?: number
          total_input_bytes?: number | null
          total_output_bytes?: number | null
          updated_at?: string | null
          webhook_triggers?: number | null
          workflow_id: string
          workflow_name: string
        }
        Update: {
          api_triggers?: number | null
          avg_activities_per_execution?: number | null
          avg_duration_ms?: number | null
          cancelled_executions?: number
          date?: string
          failed_executions?: number
          id?: string
          manual_triggers?: number | null
          max_duration_ms?: number | null
          p50_duration_ms?: number | null
          p95_duration_ms?: number | null
          p99_duration_ms?: number | null
          project_id?: string
          schedule_triggers?: number | null
          successful_executions?: number
          timeout_executions?: number
          total_activities_executed?: number | null
          total_duration_ms?: number | null
          total_executions?: number
          total_input_bytes?: number | null
          total_output_bytes?: number | null
          updated_at?: string | null
          webhook_triggers?: number | null
          workflow_id?: string
          workflow_name?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_usage_daily_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "workflow_usage_daily_workflow_id_fkey"
            columns: ["workflow_id"]
            isOneToOne: false
            referencedRelation: "workflows"
            referencedColumns: ["id"]
          },
        ]
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
      workflow_workers: {
        Row: {
          avg_task_duration_ms: number | null
          cpu_usage_percent: number | null
          created_at: string
          host: string | null
          id: string
          last_heartbeat: string | null
          memory_usage_mb: number | null
          metadata: Json | null
          port: number | null
          process_id: string | null
          project_id: string
          started_at: string | null
          status: string
          stopped_at: string | null
          task_queue_name: string
          total_tasks_completed: number | null
          total_tasks_failed: number | null
          updated_at: string
          worker_id: string
        }
        Insert: {
          avg_task_duration_ms?: number | null
          cpu_usage_percent?: number | null
          created_at?: string
          host?: string | null
          id?: string
          last_heartbeat?: string | null
          memory_usage_mb?: number | null
          metadata?: Json | null
          port?: number | null
          process_id?: string | null
          project_id: string
          started_at?: string | null
          status: string
          stopped_at?: string | null
          task_queue_name: string
          total_tasks_completed?: number | null
          total_tasks_failed?: number | null
          updated_at?: string
          worker_id: string
        }
        Update: {
          avg_task_duration_ms?: number | null
          cpu_usage_percent?: number | null
          created_at?: string
          host?: string | null
          id?: string
          last_heartbeat?: string | null
          memory_usage_mb?: number | null
          metadata?: Json | null
          port?: number | null
          process_id?: string | null
          project_id?: string
          started_at?: string | null
          status?: string
          stopped_at?: string | null
          task_queue_name?: string
          total_tasks_completed?: number | null
          total_tasks_failed?: number | null
          updated_at?: string
          worker_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "workflow_workers_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
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
          deployment_status: string | null
          description: string | null
          display_name: string
          end_with_parent: boolean | null
          execution_timeout_seconds: number | null
          id: string
          is_archived: boolean
          is_scheduled: boolean
          kebab_name: string | null
          last_run_at: string | null
          max_concurrent_executions: number | null
          max_runs: number | null
          name: string
          next_run_at: string | null
          parent_workflow_id: string | null
          project_id: string
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
          deployment_status?: string | null
          description?: string | null
          display_name: string
          end_with_parent?: boolean | null
          execution_timeout_seconds?: number | null
          id?: string
          is_archived?: boolean
          is_scheduled?: boolean
          kebab_name?: string | null
          last_run_at?: string | null
          max_concurrent_executions?: number | null
          max_runs?: number | null
          name: string
          next_run_at?: string | null
          parent_workflow_id?: string | null
          project_id: string
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
          deployment_status?: string | null
          description?: string | null
          display_name?: string
          end_with_parent?: boolean | null
          execution_timeout_seconds?: number | null
          id?: string
          is_archived?: boolean
          is_scheduled?: boolean
          kebab_name?: string | null
          last_run_at?: string | null
          max_concurrent_executions?: number | null
          max_runs?: number | null
          name?: string
          next_run_at?: string | null
          parent_workflow_id?: string | null
          project_id?: string
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
            foreignKeyName: "workflows_project_id_fkey"
            columns: ["project_id"]
            isOneToOne: false
            referencedRelation: "projects"
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
      get_activity_performance: {
        Args: { p_end_date: string; p_project_id: string; p_start_date: string }
        Returns: {
          avg_duration_ms: number
          component_name: string
          component_type: string
          p95_duration_ms: number
          success_rate: number
          total_executions: number
          total_failures: number
        }[]
      }
      increment_activity_usage: {
        Args: { activity_name: string }
        Returns: undefined
      }
      increment_code_stats: {
        Args: {
          p_duration_ms: number
          p_success: boolean
          p_workflow_id: string
        }
        Returns: undefined
      }
      increment_project_stats: {
        Args: { p_duration_ms: number; p_project_id: string }
        Returns: undefined
      }
      record_component_metric: {
        Args: {
          p_attempt_number: number
          p_completed_at: string
          p_component_id: string
          p_component_name: string
          p_component_type: string
          p_duration_ms: number
          p_error_type?: string
          p_execution_id: string
          p_is_retry: boolean
          p_metadata?: Json
          p_node_id: string
          p_project_id: string
          p_started_at: string
          p_status: string
          p_workflow_id: string
        }
        Returns: string
      }
      record_resource_event: {
        Args: {
          p_completed_at?: string
          p_completion_tokens?: number
          p_component_metric_id: string
          p_direction: string
          p_duration_ms: number
          p_error_type?: string
          p_execution_id: string
          p_latency_ms: number
          p_metadata?: Json
          p_model_name?: string
          p_operation: string
          p_project_id: string
          p_prompt_tokens?: number
          p_request_size_bytes: number
          p_resource_id: string
          p_resource_name: string
          p_resource_subtype: string
          p_resource_type: string
          p_response_size_bytes: number
          p_started_at?: string
          p_status: string
          p_target_project_id?: string
          p_target_service?: string
          p_total_tokens?: number
          p_workflow_id: string
        }
        Returns: string
      }
      record_workflow_execution_metric: {
        Args: {
          p_activity_count: number
          p_completed_at?: string
          p_duration_ms: number
          p_error_type?: string
          p_execution_id: string
          p_input_size_bytes: number
          p_metadata?: Json
          p_output_size_bytes: number
          p_project_id: string
          p_started_at?: string
          p_status: string
          p_task_queue_name: string
          p_temporal_run_id: string
          p_temporal_workflow_id: string
          p_trigger_source: string
          p_trigger_type: string
          p_workflow_id: string
          p_workflow_name: string
          p_workflow_version: string
        }
        Returns: string
      }
      update_activity_stats: {
        Args: {
          p_activity_name: string
          p_duration_ms: number
          p_project_id: string
          p_success: boolean
        }
        Returns: undefined
      }
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
