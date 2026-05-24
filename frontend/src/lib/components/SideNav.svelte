<script lang="ts">
  import { navItems, navigate, type RouteId } from "../router";

  export let route: RouteId;

  const groups = Array.from(
    navItems.reduce((map, item) => {
      const entries = map.get(item.group) || [];
      entries.push(item);
      map.set(item.group, entries);
      return map;
    }, new Map<string, typeof navItems>()),
  );
</script>

<aside class="sidebar">
  <div class="sidebar-brand">
    <div class="sidebar-logo"></div>
    <div>
      <strong>TSSP</strong>
      <span>Local Cloud OS</span>
    </div>
  </div>

  {#each groups as [group, items]}
    <section class="nav-group">
      <div class="nav-group-label">{group}</div>
      {#each items as item}
        <button
          type="button"
          class:active={route === item.id}
          class="nav-link"
          on:click={() => navigate(item.id)}
        >
          <span class="nav-link-title">{item.label}</span>
          <span class="nav-link-subtitle">{item.subtitle}</span>
        </button>
      {/each}
    </section>
  {/each}
</aside>
