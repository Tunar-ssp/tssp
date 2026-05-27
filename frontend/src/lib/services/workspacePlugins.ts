/**
 * Workspace Plugin System
 *
 * Allows extending workspace with plugins for custom functionality
 */

export interface Plugin {
  id: string;
  name: string;
  version: string;
  description: string;
  enabled: boolean;
  hooks?: PluginHooks;
}

export interface PluginHooks {
  onFileCreate?: (path: string, content: string) => void | Promise<void>;
  onFileDelete?: (path: string) => void | Promise<void>;
  onFileSave?: (path: string, content: string) => void | Promise<void>;
  onFileRename?: (oldPath: string, newPath: string) => void | Promise<void>;
  onEditorInit?: (editorInstance: any) => void | Promise<void>;
  beforeSearch?: (query: string, options: any) => string;
  afterSearch?: (results: any[]) => any[];
  onWorkspaceCreate?: (workspaceId: string) => void | Promise<void>;
  onWorkspaceDelete?: (workspaceId: string) => void | Promise<void>;
}

class PluginManager {
  private plugins: Map<string, Plugin> = new Map();
  private pluginOrder: string[] = [];

  register(plugin: Plugin): void {
    if (this.plugins.has(plugin.id)) {
      console.warn(`Plugin ${plugin.id} is already registered`);
      return;
    }
    this.plugins.set(plugin.id, plugin);
    this.pluginOrder.push(plugin.id);
  }

  unregister(pluginId: string): boolean {
    if (!this.plugins.has(pluginId)) return false;
    this.plugins.delete(pluginId);
    this.pluginOrder = this.pluginOrder.filter((id) => id !== pluginId);
    return true;
  }

  enable(pluginId: string): boolean {
    const plugin = this.plugins.get(pluginId);
    if (!plugin) return false;
    plugin.enabled = true;
    return true;
  }

  disable(pluginId: string): boolean {
    const plugin = this.plugins.get(pluginId);
    if (!plugin) return false;
    plugin.enabled = false;
    return true;
  }

  getPlugin(pluginId: string): Plugin | undefined {
    return this.plugins.get(pluginId);
  }

  getAllPlugins(): Plugin[] {
    return Array.from(this.plugins.values());
  }

  getEnabledPlugins(): Plugin[] {
    return this.getAllPlugins().filter((p) => p.enabled);
  }

  async executeHook(
    hookName: keyof PluginHooks,
    ...args: any[]
  ): Promise<void> {
    for (const pluginId of this.pluginOrder) {
      const plugin = this.plugins.get(pluginId);
      if (!plugin || !plugin.enabled || !plugin.hooks) continue;

      const hook = plugin.hooks[hookName];
      if (hook) {
        try {
          await (hook as any)(...args);
        } catch (error) {
          console.error(`Error executing hook ${hookName} in plugin ${pluginId}:`, error);
        }
      }
    }
  }

  executeFilterHook(
    hookName: 'beforeSearch' | 'afterSearch',
    initialValue: any,
    ...args: any[]
  ): any {
    let value = initialValue;

    for (const pluginId of this.pluginOrder) {
      const plugin = this.plugins.get(pluginId);
      if (!plugin || !plugin.enabled || !plugin.hooks) continue;

      const hook = plugin.hooks[hookName];
      if (hook) {
        try {
          value = (hook as any)(value, ...args);
        } catch (error) {
          console.error(`Error executing filter hook ${hookName} in plugin ${pluginId}:`, error);
        }
      }
    }

    return value;
  }
}

export const pluginManager = new PluginManager();

// Built-in plugins
export const builtInPlugins = {
  typescript: {
    id: 'typescript-support',
    name: 'TypeScript Support',
    version: '1.0.0',
    description: 'Syntax highlighting and basic IntelliSense for TypeScript',
    enabled: true,
  } as Plugin,

  markdown: {
    id: 'markdown-support',
    name: 'Markdown Support',
    version: '1.0.0',
    description: 'Markdown preview and formatting',
    enabled: true,
  } as Plugin,

  eslint: {
    id: 'eslint-integration',
    name: 'ESLint Integration',
    version: '1.0.0',
    description: 'Real-time linting for JavaScript/TypeScript',
    enabled: false,
  } as Plugin,

  prettier: {
    id: 'prettier-integration',
    name: 'Prettier Integration',
    version: '1.0.0',
    description: 'Code formatting with Prettier',
    enabled: false,
  } as Plugin,
};

// Register built-in plugins
Object.values(builtInPlugins).forEach((plugin) => {
  pluginManager.register(plugin);
});
