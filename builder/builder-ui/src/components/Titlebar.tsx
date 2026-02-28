import { useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './Titlebar.css'

interface Props {
  projectName: string
  projectPath: string | null
  isDirty: boolean
  onNew: () => void
  onOpen: () => void
  onSave: () => void
  onSaveAs: () => void
}

const SPLASH_MIN_MS = 2200

export default function Titlebar({ projectName, projectPath, isDirty, onNew, onOpen, onSave, onSaveAs }: Props) {
  useEffect(() => {
    const startTime = Date.now()
    const dismiss = () => {
      if ((window as any).__dismissSplash) (window as any).__dismissSplash()
    }
    invoke('shell_ready')
      .then(() => {
        const remaining = Math.max(0, SPLASH_MIN_MS - (Date.now() - startTime))
        setTimeout(dismiss, remaining)
      })
      .catch(() => {
        const remaining = Math.max(0, SPLASH_MIN_MS - (Date.now() - startTime))
        setTimeout(dismiss, remaining)
      })
  }, [])

  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-left" data-tauri-drag-region>
        <span className="titlebar-appname">Lynx Builder</span>
        <div className="titlebar-divider" />
        <span className="titlebar-project">
          {projectName || 'Untitled'}
          {isDirty && <span className="titlebar-dirty">●</span>}
        </span>
        {projectPath && <span className="titlebar-path">{projectPath}</span>}
      </div>
      <div className="titlebar-actions">
        <button className="tb-btn" onClick={onNew}>New</button>
        <button className="tb-btn" onClick={onOpen}>Open</button>
        <button className="tb-btn tb-btn-save" onClick={onSave}>
          {isDirty ? 'Save●' : 'Save'}
        </button>
        <button className="tb-btn" onClick={onSaveAs}>Save As</button>
        <div className="wc-group">
          <button className="wc-btn" onClick={() => invoke('shell_minimize')} title="Minimize">
            <svg width="10" height="2" viewBox="0 0 10 2"><rect width="10" height="2" rx="1" fill="currentColor"/></svg>
          </button>
          <button className="wc-btn" onClick={() => invoke('shell_maximize')} title="Maximize">
            <svg width="10" height="10" viewBox="0 0 10 10"><rect x="1" y="1" width="8" height="8" rx="1" stroke="currentColor" strokeWidth="1.5" fill="none"/></svg>
          </button>
          <button className="wc-btn wc-close" onClick={() => invoke('shell_close')} title="Close">
            <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/></svg>
          </button>
        </div>
      </div>
    </div>
  )
}