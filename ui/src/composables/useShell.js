import { ref, nextTick } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'

export function useShell() {
  const shellService = ref(null)
  const termEl       = ref(null)
  let term     = null
  let termWs   = null
  let fitAddon = null
  let resizeObserver = null
  let resizeFrame = null
  let pasteHandler = null
  let keyHandler = null

  function sendResize() {
    if (term && termWs?.readyState === WebSocket.OPEN) {
      termWs.send(JSON.stringify({ type: 'resize', cols: term.cols, rows: term.rows }))
    }
  }

  function fitShell() {
    if (!term || !fitAddon) return
    if (resizeFrame) cancelAnimationFrame(resizeFrame)
    resizeFrame = requestAnimationFrame(() => {
      resizeFrame = null
      if (!term || !fitAddon) return
      fitAddon.fit()
      sendResize()
    })
  }

  function sendPaste(text) {
    if (termWs?.readyState === WebSocket.OPEN) {
      termWs.send(text.replace(/\r?\n/g, '\r'))
    }
  }

  async function openShell(project, service) {
    closeShell()
    shellService.value = service
    await nextTick()

    term = new Terminal({
      theme: { background: '#0b0e16', foreground: '#b8c5d8', cursor: '#6366f1' },
      fontFamily: 'IBM Plex Mono, monospace',
      fontSize: 13,
      cursorBlink: true,
    })
    fitAddon = new FitAddon()
    term.loadAddon(fitAddon)
    term.open(termEl.value)
    fitShell()
    term.focus()

    resizeObserver = new ResizeObserver(() => fitShell())
    resizeObserver.observe(termEl.value)
    window.addEventListener('resize', fitShell)

    pasteHandler = (e) => {
      const text = e.clipboardData?.getData('text')
      if (!text) return
      e.preventDefault()
      e.stopPropagation()
      e.stopImmediatePropagation()
      sendPaste(text)
    }
    termEl.value.addEventListener('paste', pasteHandler, true)

    keyHandler = (e) => {
      if (!e.ctrlKey || e.altKey || e.metaKey) return

      const key = e.key.toLowerCase()
      if (key === 'c' && term?.hasSelection()) {
        e.preventDefault()
        e.stopPropagation()
        e.stopImmediatePropagation()
        navigator.clipboard?.writeText(term.getSelection())
        term.clearSelection()
        return false
      }

      if (key === 'v' && !e.shiftKey) {
        e.stopPropagation()
        e.stopImmediatePropagation()
        return false
      }
    }
    term.attachCustomKeyEventHandler((e) => keyHandler(e) !== false)

    const proto = location.protocol === 'https:' ? 'wss' : 'ws'
    const url   = `${proto}://${location.host}/api/projects/${project}/shell?service=${encodeURIComponent(service)}`
    termWs = new WebSocket(url)
    termWs.binaryType = 'arraybuffer'

    termWs.onopen = () => fitShell()
    termWs.onmessage = (e) => {
      if (e.data instanceof ArrayBuffer) term.write(new Uint8Array(e.data))
      else term.write(e.data)
    }
    termWs.onclose = () => term?.write('\r\n\x1b[31m[disconnected]\x1b[0m\r\n')

    term.onData((data) => {
      if (termWs?.readyState === WebSocket.OPEN) termWs.send(data)
    })
    term.onResize(({ cols, rows }) => {
      if (termWs?.readyState === WebSocket.OPEN)
        termWs.send(JSON.stringify({ type: 'resize', cols, rows }))
    })
  }

  function closeShell() {
    if (resizeFrame) cancelAnimationFrame(resizeFrame)
    resizeFrame = null
    resizeObserver?.disconnect()
    resizeObserver = null
    window.removeEventListener('resize', fitShell)
    if (termEl.value && pasteHandler) termEl.value.removeEventListener('paste', pasteHandler, true)
    pasteHandler = null
    keyHandler = null
    termWs?.close()
    termWs = null
    term?.dispose()
    term = null
    shellService.value = null
  }

  return { shellService, termEl, openShell, closeShell, fitShell }
}
