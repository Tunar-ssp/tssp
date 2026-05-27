<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Task {
    id: string;
    title: string;
    description?: string;
    status: 'todo' | 'in-progress' | 'done';
    priority: 'low' | 'medium' | 'high';
    assignee?: string;
    dueDate?: number;
    tags?: string[];
    createdAt: number;
  }

  interface $$Props {
    tasks?: Task[];
    onCreateTask?: (task: Omit<Task, 'id' | 'createdAt'>) => void;
    onUpdateTask?: (id: string, updates: Partial<Task>) => void;
    onDeleteTask?: (id: string) => void;
  }

  let {
    tasks = [],
    onCreateTask = () => {},
    onUpdateTask = () => {},
    onDeleteTask = () => {},
  }: $$Props = $props();

  let newTaskTitle = $state('');
  let filterStatus = $state<'all' | 'todo' | 'in-progress' | 'done'>('all');
  let showNewTaskForm = $state(false);

  let filteredTasks = $derived.by(() => {
    if (filterStatus === 'all') {
      return tasks;
    }
    return tasks.filter((t) => t.status === filterStatus);
  });

  let taskStats = $derived.by(() => ({
    total: tasks.length,
    todo: tasks.filter((t) => t.status === 'todo').length,
    inProgress: tasks.filter((t) => t.status === 'in-progress').length,
    done: tasks.filter((t) => t.status === 'done').length,
  }));

  function handleCreateTask() {
    if (!newTaskTitle.trim()) return;
    onCreateTask({
      title: newTaskTitle,
      status: 'todo',
      priority: 'medium',
    });
    newTaskTitle = '';
    showNewTaskForm = false;
  }

  function getPriorityColor(priority: string) {
    switch (priority) {
      case 'high':
        return 'var(--danger)';
      case 'medium':
        return 'var(--orange)';
      case 'low':
        return 'var(--green)';
      default:
        return 'var(--text-2)';
    }
  }

  function getStatusIcon(status: string) {
    switch (status) {
      case 'todo':
        return Icons.Circle;
      case 'in-progress':
        return Icons.Clock;
      case 'done':
        return Icons.CheckCircle;
      default:
        return Icons.Circle;
    }
  }

  function formatDate(timestamp?: number): string {
    if (!timestamp) return 'No due date';
    const date = new Date(timestamp);
    return date.toLocaleDateString();
  }
</script>

<div class="task-panel">
  <div class="panel-header">
    <h3>Tasks</h3>
    <button
      type="button"
      class="create-btn"
      onclick={() => (showNewTaskForm = true)}
      title="New task"
    >
      <Icons.Plus size={14} />
    </button>
  </div>

  <div class="task-stats">
    <div class="stat">
      <span class="stat-value">{taskStats.total}</span>
      <span class="stat-label">Total</span>
    </div>
    <div class="stat">
      <span class="stat-value">{taskStats.todo}</span>
      <span class="stat-label">Todo</span>
    </div>
    <div class="stat">
      <span class="stat-value">{taskStats.inProgress}</span>
      <span class="stat-label">In Progress</span>
    </div>
    <div class="stat">
      <span class="stat-value">{taskStats.done}</span>
      <span class="stat-label">Done</span>
    </div>
  </div>

  <div class="filter-tabs">
    {#each ['all', 'todo', 'in-progress', 'done'] as status (status)}
      <button
        type="button"
        class="filter-tab"
        class:active={filterStatus === status}
        onclick={() => (filterStatus = status)}
      >
        {status.replace('-', ' ')}
      </button>
    {/each}
  </div>

  {#if filteredTasks.length === 0}
    <div class="empty-state">
      <Icons.CheckCircle size={24} />
      <p>No tasks</p>
    </div>
  {:else}
    <div class="tasks-list">
      {#each filteredTasks as task (task.id)}
        <div class="task-item">
          <div class="task-head">
            <button
              type="button"
              class="task-checkbox"
              onclick={() =>
                onUpdateTask(task.id, {
                  status: task.status === 'done' ? 'todo' : 'done',
                })}
              title={task.status === 'done' ? 'Mark as todo' : 'Mark as done'}
            >
              <svelte:component this={getStatusIcon(task.status)} size={16} />
            </button>
            <span class="task-title">{task.title}</span>
            <span class="task-priority" style="color: {getPriorityColor(task.priority)}">
              ●
            </span>
          </div>
          {#if task.dueDate}
            <div class="task-meta">
              <Icons.Calendar size={12} />
              <span>{formatDate(task.dueDate)}</span>
            </div>
          {/if}
          <div class="task-actions">
            <button
              type="button"
              class="action-btn"
              onclick={() => onDeleteTask(task.id)}
              title="Delete task"
            >
              <Icons.Trash2 size={12} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if showNewTaskForm}
    <div class="new-task-form">
      <input
        type="text"
        placeholder="New task..."
        bind:value={newTaskTitle}
        onkeydown={(e) => {
          if (e.key === 'Enter') handleCreateTask();
          if (e.key === 'Escape') (showNewTaskForm = false);
        }}
        autofocus
      />
      <div class="form-actions">
        <button type="button" class="btn btn-primary" onclick={handleCreateTask}>
          Create
        </button>
        <button type="button" class="btn btn-secondary" onclick={() => (showNewTaskForm = false)}>
          Cancel
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .task-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .panel-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text);
  }

  .create-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .create-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .task-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    padding: 8px;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 8px;
    border-radius: 6px;
    background: var(--surface);
  }

  .stat-value {
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .stat-label {
    font-size: 10px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .filter-tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .filter-tab {
    flex: 1;
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    border-bottom: 2px solid transparent;
  }

  .filter-tab:hover {
    color: var(--text);
  }

  .filter-tab.active {
    color: var(--blue);
    border-bottom-color: var(--blue);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 40px 16px;
    color: var(--muted);
    text-align: center;
  }

  .tasks-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .task-item {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    background: var(--bg);
    border-radius: 8px;
    border: 1px solid var(--hairline);
  }

  .task-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .task-checkbox {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .task-checkbox:hover {
    color: var(--text);
  }

  .task-title {
    flex: 1;
    font-size: 13px;
    color: var(--text);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .task-priority {
    font-size: 14px;
    flex-shrink: 0;
  }

  .task-meta {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--muted);
  }

  .task-actions {
    display: flex;
    gap: 4px;
    justify-content: flex-end;
  }

  .action-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .action-btn:hover {
    color: var(--danger);
  }

  .new-task-form {
    padding: 12px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .new-task-form input {
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font-size: 13px;
  }

  .new-task-form input:focus {
    outline: none;
    border-color: var(--blue);
  }

  .form-actions {
    display: flex;
    gap: 6px;
  }

  .btn {
    flex: 1;
    padding: 6px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    cursor: pointer;
    font-size: 11px;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .btn-primary {
    background: var(--blue-soft);
    border-color: var(--blue);
    color: var(--blue);
  }

  .btn-primary:hover {
    background: var(--blue);
    color: white;
  }

  .btn-secondary:hover {
    background: var(--surface-2);
  }
</style>
