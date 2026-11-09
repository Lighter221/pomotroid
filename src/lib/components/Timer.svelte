<script lang="ts">
  // Orchestrator component. Subscribes to timer events, owns keyboard listener,
  // and renders TimerDial + TimerDisplay + TimerFooter.
  import { onMount } from 'svelte';
  import {
    timerToggle,
    timerRestartRound,
    timerStopAlert,
    timerSkip,
    getTimerState,
    onTimerTick,
    onTimerPaused,
    onTimerResumed,
    onRoundChange,
    onTimerReset,
  } from '$lib/ipc';
  import { timerState } from '$lib/stores/timer';
  import { settings } from '$lib/stores/settings';
  import { fade } from 'svelte/transition';
  import TimerDial from './TimerDial.svelte';
  import TimerDisplay from './TimerDisplay.svelte';
  import TimerFooter from './TimerFooter.svelte';
  import MiniControls from './MiniControls.svelte';
  import Tooltip from './Tooltip.svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import * as m from '$paraglide/messages.js';
  import { notificationShow } from '$lib/ipc';

  interface Props {
    isCompact?: boolean;
    uiScale?: number;
  }

  let { isCompact = false, uiScale = 1 }: Props = $props();

  let state = $derived($timerState);

  function roundColor(rt: string): string {
    if (rt === 'work') return 'var(--color-focus-round)';
    if (rt === 'short-break') return 'var(--color-short-round)';
    return 'var(--color-long-round)';
  }

  function roundLabel(rt: string, isAlerting?: boolean): string {
    if (isAlerting) return '⏰ 时间到！点击停止提醒';
    return m.round_label_work();
  }

  onMount(() => {
    const cleanups: UnlistenFn[] = [];

    // Async setup: hydrate state and register event listeners.
    (async () => {
      const initial = await getTimerState();
      timerState.set(initial);

      cleanups.push(
        await onTimerTick(({ elapsed_secs, total_secs }) => {
          timerState.update((s) => ({
            ...s,
            elapsed_secs,
            total_secs,
            is_running: true,
            is_paused: false,
            is_alerting: false,
          }));
        }),
        await onTimerPaused(({ elapsed_secs }) => {
          timerState.update((s) => ({
            ...s,
            elapsed_secs,
            is_running: false,
            is_paused: true,
          }));
        }),
        await onTimerResumed(({ elapsed_secs }) => {
          timerState.update((s) => ({
            ...s,
            elapsed_secs,
            is_running: true,
            is_paused: false,
          }));
        }),
        await onRoundChange((snap) => {
          timerState.set(snap);
          if ($settings.notifications_enabled || snap.is_alerting) {
            const title = '⏰ 工作时长已到！';
            const body = '请休息一下，点击窗口停止提醒。';
            notificationShow(title, body).catch(() => {});
          }
        }),
        await onTimerReset((snap) => {
          timerState.set(snap);
        })
      );
    })();

    return () => {
      for (const unlisten of cleanups) unlisten();
    };
  });
</script>

<div class="timer-outer" class:compact={isCompact}>
  <div class="timer" style="zoom: {uiScale}">
    <!-- Dial + display stacked (display centered over dial) -->
    <div class="dial-stack">
      <TimerDial snap={state} countdown={$settings.dial_countdown} />
      <TimerDisplay {state} />
    </div>

    {#if !isCompact}
      <!-- Round type label sits below the dial as a normal flex child so it
           does not affect the dial-stack height used to centre TimerDisplay. -->
      <div
        class="round-label"
        class:label-alert={state.is_alerting}
        style="color: {state.is_alerting ? 'var(--color-accent, #ff4d4f)' : roundColor(state.round_type)}"
      >
        {roundLabel(state.round_type, state.is_alerting)}
      </div>

      <div class="controls-wrapper">
        <!-- Back: restart current round -->
        <Tooltip text={m.tooltip_restart_round()}>
          <button class="btn-side" onclick={timerRestartRound} aria-label="Restart round">
            <svg width="18" height="18" viewBox="0 0 16 16">
              <polygon points="15,1 6,8 15,15" fill="currentColor" />
              <rect x="1" y="1" width="3" height="14" rx="1" fill="currentColor" />
            </svg>
          </button>
        </Tooltip>

        {#if state.is_alerting}
          <!-- Alerting: click to stop alert -->
          <button
            class="play-pause btn-alerting"
            onclick={timerStopAlert}
            aria-label="Stop Alert"
            title="停止提醒并重置"
          >
            <span class="icon">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </span>
          </button>
        {:else}
          <!-- Play / Pause — icon fades when state changes -->
          <button
            class="play-pause"
            onclick={timerToggle}
            aria-label={state.is_running ? 'Pause' : 'Play'}
          >
            {#key state.is_running}
              <span class="icon" in:fade={{ duration: 120 }}>
                {#if state.is_running}
                  <svg width="24" height="24" viewBox="0 0 24 24">
                    <rect x="5" y="3" width="5" height="18" rx="1.5" fill="currentColor" />
                    <rect x="14" y="3" width="5" height="18" rx="1.5" fill="currentColor" />
                  </svg>
                {:else}
                  <svg width="18" height="18" viewBox="0 0 24 24" style="overflow: visible;">
                    <polygon points="4,0 28,12 4,24" fill="currentColor" />
                  </svg>
                {/if}
              </span>
            {/key}
          </button>
        {/if}

        <TimerFooter snap={state} />
      </div>
    {/if}
  </div>

  {#if isCompact}
    <MiniControls />
  {/if}
</div>

<style>
  .timer-outer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .timer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .dial-stack {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .controls-wrapper {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 4px 12px;
  }
  .controls-wrapper > :global(*) {
    aspect-ratio: 1;
  }

  .btn-side {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-foreground-darker, var(--color-foreground));
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 4px;
    transition:
      color var(--transition-default),
      background var(--transition-default);
  }

  .btn-side:hover {
    color: var(--color-foreground);
    background: var(--color-hover);
  }

  .play-pause {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-foreground);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: 50%;
    border: 2px solid var(--color-foreground-darker, var(--color-foreground));
    transition:
      color var(--transition-default),
      border-color var(--transition-default),
      background var(--transition-default);
    overflow: hidden; /* clip the fading icon within the circle */
  }

  .play-pause:hover {
    color: var(--color-accent);
    border-color: var(--color-accent);
    background: var(--color-hover);
  }

  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .round-label {
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    /* Collapse the gap above: the flex gap already provides spacing from the dial. */
    margin-top: -8px;
  }

  .label-alert {
    animation: alert-pulse 1s infinite alternate;
  }

  .btn-alerting {
    border-color: #ff4d4f !important;
    color: #ff4d4f !important;
    animation: alert-pulse 1s infinite alternate;
  }

  @keyframes alert-pulse {
    0% {
      transform: scale(1);
      opacity: 0.85;
    }
    100% {
      transform: scale(1.08);
      opacity: 1;
      filter: drop-shadow(0 0 6px rgba(255, 77, 79, 0.6));
    }
  }
</style>
