<script>
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { cubicOut, cubicIn } from 'svelte/easing';
  import { store } from './lib/store.svelte';
  import NavBar from './components/NavBar.svelte';
  import Toast from './components/Toast.svelte';
  import StatusTab from './routes/StatusTab.svelte';
  import ConfigTab from './routes/ConfigTab.svelte';
  import ModulesTab from './routes/ModulesTab.svelte';
  import LogsTab from './routes/LogsTab.svelte';
  import InfoTab from './routes/InfoTab.svelte';
  import './app.css';
  import './layout.css';
  
  // Default tab is 'status'
  let activeTab = $state('status');
  let transitionDirection = $state(1);
  let touchStartX = 0;
  let touchStartY = 0;
  let touchEndX = 0;
  let touchEndY = 0;

  // Warning modal state
  let showWarning = $state(false);
  let countdown = $state(5);
  let canClose = $state(false);

  const TABS = ['status', 'config', 'modules', 'logs', 'info'];
  function switchTab(id) {
    const currentIndex = TABS.indexOf(activeTab);
    const newIndex = TABS.indexOf(id);
    if (currentIndex === newIndex) return;
    transitionDirection = newIndex > currentIndex ? 1 : -1;
    activeTab = id;
  }

  function handleTouchStart(e) {
    touchStartX = e.changedTouches[0].screenX;
    touchStartY = e.changedTouches[0].screenY;
  }

  function handleTouchEnd(e) {
    touchEndX = e.changedTouches[0].screenX;
    touchEndY = e.changedTouches[0].screenY;
    
    const threshold = 50;
    const diffX = touchStartX - touchEndX;
    const diffY = touchStartY - touchEndY;
    
    // 如果垂直滑动距离大于水平滑动距离,则不翻页(用户是在滚动内容)
    if (Math.abs(diffY) > Math.abs(diffX)) return;
    
    // 水平滑动距离太小也不翻页
    if (Math.abs(diffX) < threshold) return;
    
    const currentIndex = TABS.indexOf(activeTab);
    
    if (diffX > 0 && currentIndex < TABS.length - 1) {
      switchTab(TABS[currentIndex + 1]);
    } else if (diffX < 0 && currentIndex > 0) {
      switchTab(TABS[currentIndex - 1]);
    }
  }

  onMount(() => {
    store.init();

    // Check for warning
    const warningShown = localStorage.getItem('hymo_warning_shown');
    if (!warningShown) {
      showWarning = true;
      const timer = setInterval(() => {
        countdown--;
        if (countdown <= 0) {
          clearInterval(timer);
          canClose = true;
        }
      }, 1000);
    }
  });

  function closeWarning() {
    if (!canClose) return;
    localStorage.setItem('hymo_warning_shown', 'true');
    showWarning = false;
  }
</script>

<div class="app-root" 
     style:background-image={store.backgroundImage ? `url('${store.backgroundImage}')` : 'none'} 
     style:background-size="cover" 
     style:background-position="center"
     style:background-attachment="fixed">
  <NavBar {activeTab} onTabChange={switchTab} />

  <main class="main-content" ontouchstart={handleTouchStart} ontouchend={handleTouchEnd}>
    {#key activeTab}
      <div class="tab-pane" 
           in:fly={{ x: 30 * transitionDirection, duration: 250, delay: 90, easing: cubicOut }} 
           out:fly={{ x: -30 * transitionDirection, duration: 150, easing: cubicIn }}>
        
        {#if activeTab === 'status'}
          <StatusTab />
        {:else if activeTab === 'config'}
          <ConfigTab />
        {:else if activeTab === 'modules'}
          <ModulesTab />
        {:else if activeTab === 'logs'}
          <LogsTab />
        {:else if activeTab === 'info'}
          <InfoTab />
        {/if}
      </div>
    {/key}
  </main>

  <Toast />

  {#if showWarning}
    <div class="warning-overlay">
      <div class="warning-box">
        <h2 class="warning-title">⚠️ 警告 / Warning</h2>
        <div class="warning-content">
          <p class="warning-text-zh"><strong>HymoFS 是一个实验性项目。</strong></p>
          <p class="warning-text-zh">它可能会导致手机性能下降，并且可能存在潜在的稳定性问题。</p>
          <div class="warning-divider"></div>
          <p class="warning-text-en"><strong>HymoFS is an experimental project.</strong></p>
          <p class="warning-text-en">It may cause performance degradation and potential stability issues.</p>
        </div>
        <button class="warning-btn" disabled={!canClose} onclick={closeWarning} class:enabled={canClose}>
          {#if canClose}
            知道了 / I understand
          {:else}
            请等待 / Please wait ({countdown}s)
          {/if}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .warning-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 9999;
    backdrop-filter: blur(5px);
  }

  .warning-box {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 16px;
    padding: 24px;
    width: 90%;
    max-width: 400px;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
    color: var(--text-primary, #fff);
    text-align: center;
  }

  .warning-title {
    margin: 0 0 20px 0;
    color: #ff4444;
    font-size: 1.5rem;
  }

  .warning-content {
    margin-bottom: 24px;
    text-align: left;
  }

  .warning-text-zh {
    margin: 8px 0;
    font-size: 1rem;
    line-height: 1.5;
  }

  .warning-text-en {
    margin: 8px 0;
    font-size: 0.9rem;
    color: var(--text-secondary, #aaa);
    line-height: 1.4;
  }

  .warning-divider {
    height: 1px;
    background: var(--border-color, #333);
    margin: 16px 0;
  }

  .warning-btn {
    width: 100%;
    padding: 12px;
    border-radius: 8px;
    border: none;
    background: var(--bg-tertiary, #333);
    color: var(--text-secondary, #888);
    font-size: 1rem;
    font-weight: bold;
    cursor: not-allowed;
    transition: all 0.3s ease;
  }

  .warning-btn.enabled {
    background: #ff4444;
    color: white;
    cursor: pointer;
  }

  .warning-btn.enabled:hover {
    background: #ff2222;
    transform: translateY(-2px);
  }
</style>