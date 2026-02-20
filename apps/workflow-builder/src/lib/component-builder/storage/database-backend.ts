/**
 * Database Storage Backend
 *
 * Stores components in a database (PostgreSQL via Prisma).
 * Suitable for multi-tenant deployments or when file system access is limited.
 *
 * Note: For high-scale hosted deployments, this should be combined with
 * edge caching as described in the hosted-component-scalability plan.
 */

import type {
  ComponentStorageBackend,
  StoredComponent,
  ListComponentsOptions,
  ListComponentsResult,
} from './types';

/**
 * Database backend configuration
 */
export interface DatabaseBackendConfig {
  /** Prisma client instance or connection string */
  prisma?: unknown; // PrismaClient type when available

  /** Table/collection name prefix */
  tablePrefix?: string;
}

/**
 * Database storage backend implementation
 *
 * This is a placeholder implementation that shows the interface.
 * The actual implementation will use Prisma when the database schema is set up.
 */
export class DatabaseStorageBackend implements ComponentStorageBackend {
  private config: DatabaseBackendConfig;
  private components: Map<string, StoredComponent> = new Map();

  constructor(config: DatabaseBackendConfig = {}) {
    this.config = config;
  }

  async initialize(): Promise<void> {
    // In production, this would:
    // 1. Connect to the database
    // 2. Run migrations if needed
    // 3. Set up connection pool

    console.log(
      'DatabaseStorageBackend initialized (in-memory mode until Prisma schema is set up)'
    );
  }

  async save(component: StoredComponent): Promise<StoredComponent> {
    const now = new Date();
    const componentToSave: StoredComponent = {
      ...component,
      createdAt: component.createdAt || now,
      updatedAt: now,
    };

    // In production with Prisma:
    // await this.prisma.component.upsert({
    //   where: { id: component.id },
    //   create: componentToSave,
    //   update: componentToSave,
    // });

    this.components.set(component.id, componentToSave);
    return componentToSave;
  }

  async get(id: string): Promise<StoredComponent | null> {
    // In production with Prisma:
    // return this.prisma.component.findUnique({ where: { id } });

    return this.components.get(id) || null;
  }

  async getVersion(
    id: string,
    version: string
  ): Promise<StoredComponent | null> {
    // In production with Prisma:
    // return this.prisma.componentVersion.findUnique({
    //   where: { componentId_version: { componentId: id, version } }
    // });

    const component = this.components.get(id);
    if (component && component.version === version) {
      return component;
    }
    return null;
  }

  async list(options?: ListComponentsOptions): Promise<ListComponentsResult> {
    const {
      category,
      status,
      createdBy,
      search,
      offset = 0,
      limit = 50,
      sortBy = 'name',
      sortOrder = 'asc',
    } = options || {};

    // In production with Prisma:
    // const where: Prisma.ComponentWhereInput = {};
    // if (category) where.category = category;
    // if (status) where.status = status;
    // if (createdBy) where.createdBy = createdBy;
    // if (search) {
    //   where.OR = [
    //     { name: { contains: search, mode: 'insensitive' } },
    //     { description: { contains: search, mode: 'insensitive' } },
    //   ];
    // }
    //
    // const [components, total] = await Promise.all([
    //   this.prisma.component.findMany({
    //     where,
    //     skip: offset,
    //     take: limit,
    //     orderBy: { [sortBy]: sortOrder },
    //   }),
    //   this.prisma.component.count({ where }),
    // ]);

    let components = Array.from(this.components.values());

    // Apply filters
    if (category) {
      components = components.filter((c) => c.category === category);
    }
    if (status) {
      components = components.filter((c) => c.status === status);
    }
    if (createdBy) {
      components = components.filter((c) => c.createdBy === createdBy);
    }
    if (search) {
      const searchLower = search.toLowerCase();
      components = components.filter(
        (c) =>
          c.name.toLowerCase().includes(searchLower) ||
          c.description.toLowerCase().includes(searchLower)
      );
    }

    // Sort
    components.sort((a, b) => {
      let comparison = 0;
      switch (sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'createdAt':
          comparison = a.createdAt.getTime() - b.createdAt.getTime();
          break;
        case 'updatedAt':
          comparison = a.updatedAt.getTime() - b.updatedAt.getTime();
          break;
        case 'usageCount':
          comparison = a.metadata.usageCount - b.metadata.usageCount;
          break;
      }
      return sortOrder === 'asc' ? comparison : -comparison;
    });

    const total = components.length;
    components = components.slice(offset, offset + limit);

    return { components, total, offset, limit };
  }

  async update(
    id: string,
    updates: Partial<StoredComponent>
  ): Promise<StoredComponent | null> {
    const existing = await this.get(id);
    if (!existing) {
      return null;
    }

    const updated: StoredComponent = {
      ...existing,
      ...updates,
      id: existing.id,
      createdAt: existing.createdAt,
      updatedAt: new Date(),
    };

    // In production with Prisma:
    // return this.prisma.component.update({
    //   where: { id },
    //   data: updates,
    // });

    this.components.set(id, updated);
    return updated;
  }

  async delete(id: string): Promise<boolean> {
    // In production with Prisma:
    // await this.prisma.component.delete({ where: { id } });

    return this.components.delete(id);
  }

  async exists(id: string): Promise<boolean> {
    // In production with Prisma:
    // const count = await this.prisma.component.count({ where: { id } });
    // return count > 0;

    return this.components.has(id);
  }

  async getVersions(id: string): Promise<string[]> {
    // In production with Prisma:
    // const versions = await this.prisma.componentVersion.findMany({
    //   where: { componentId: id },
    //   select: { version: true },
    //   orderBy: { createdAt: 'desc' },
    // });
    // return versions.map(v => v.version);

    const component = this.components.get(id);
    return component ? [component.version] : [];
  }

  async incrementUsage(id: string): Promise<void> {
    // In production with Prisma:
    // await this.prisma.component.update({
    //   where: { id },
    //   data: { usageCount: { increment: 1 } },
    // });

    const component = this.components.get(id);
    if (component) {
      component.metadata.usageCount++;
    }
  }

  async search(query: string, limit = 10): Promise<StoredComponent[]> {
    // In production with Prisma + pg_trgm:
    // return this.prisma.$queryRaw`
    //   SELECT * FROM components
    //   WHERE status = 'published'
    //   AND (
    //     name % ${query}
    //     OR description % ${query}
    //   )
    //   ORDER BY similarity(name, ${query}) DESC
    //   LIMIT ${limit}
    // `;

    const result = await this.list({
      search: query,
      limit,
      status: 'published',
    });
    return result.components;
  }
}

/**
 * Prisma schema for components (to be added to schema.prisma)
 *
 * model Component {
 *   id          String   @id
 *   name        String
 *   description String
 *   category    String
 *   temporalType String
 *   version     String
 *   status      String   @default("draft")
 *
 *   // Artifacts stored as JSON
 *   artifacts   Json
 *   schema      Json
 *   metadata    Json
 *
 *   createdAt   DateTime @default(now())
 *   updatedAt   DateTime @updatedAt
 *   createdBy   String
 *
 *   // For multi-tenant
 *   accountId   String?
 *   account     Account? @relation(fields: [accountId], references: [id])
 *
 *   // Indexes
 *   @@index([category])
 *   @@index([status])
 *   @@index([createdBy])
 *   @@index([accountId])
 *
 *   // Full-text search (PostgreSQL)
 *   // @@index([name, description], type: GIN, ops: raw("gin_trgm_ops"))
 * }
 *
 * model ComponentVersion {
 *   id          String    @id @default(cuid())
 *   componentId String
 *   version     String
 *   artifacts   Json
 *   schema      Json
 *   createdAt   DateTime  @default(now())
 *
 *   component   Component @relation(fields: [componentId], references: [id])
 *
 *   @@unique([componentId, version])
 * }
 */
