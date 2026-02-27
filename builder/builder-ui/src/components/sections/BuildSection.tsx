import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { LynxProject } from '../../App'

interface Props {
  project: LynxProject
  projectPath: string | null
}

interface BuildLog {
  type: 'info' | 'success' | 'error' | 'progress'
  message: string
  time: string
}

export default function BuildSection({ project, projectPath }: Props) {
  const [building, setBuilding] = useState(false)
  const [logs, setLogs] = useState<BuildLog[]>([])
  const [result, setResult] = useState<null | { success: boolean; file_count?: number; total_bytes?: number; output_path?: string; error?: string }>(null)

  const addLog = (type: BuildLog['type'], message: string) => {
    const time = new Date().toLocaleTimeString('en', { hour12: false })
    setLogs(prev => [...prev, { type, message, time }])
  }

  const validate = () => {
    const errors: string[] = []
    if (!project.app.name.trim()) errors.push('App name is required')
    if (!project.app.version.trim()) errors.push('Version is required')
    if (!project.app.id.trim()) errors.push('App ID is required')
    if (project.files.length === 0) errors.push('No files added to bundle')
    if (project.steps.length === 0) errors.push('No install steps defined')
    return errors
  }

  const handleBuild = async () => {
    if (!projectPath) {
      addLog('error', 'Save the project first before building')
      return
    }

    const errors = validate()
    if (errors.length > 0) {
      errors.forEach(e => addLog('error', e))
      return
    }

    setBuilding(true)
    setLogs([])
    setResult(null)

    addLog('info', `Building ${project.app.name} v${project.app.version}...`)

    // Listen to build progress events from the Rust backend
    const unlisten = await listen<string>('build-progress', event => {
      try {
        const data = JSON.parse(event.payload)
        if (data.type === 'file_progress') {
          addLog('progress', `  ${data.file_name}`)
        } else if (data.type === 'step_begin') {
          addLog('info', `→ ${data.step_label}`)
        } else if (data.type === 'warning') {
          addLog('info', `⚠ ${data.message}`)
        }
      } catch {}
    })

    try {
      const outputPath = projectPath.replace(/\.lynx$/, '.lynxpak')
      addLog('info', `Output: ${outputPath}`)

      const res = await invoke<typeof result>('build_pak', {
        projectPath,
        outputPath,
      })

      setResult(res)
      if (res?.success) {
        addLog('success', `✅ Build complete! ${res.file_count} files, ${((res.total_bytes || 0) / 1024).toFixed(1)} KB`)
        addLog('success', `Output: ${res.output_path}`)
      }
    } catch (e: any) {
      addLog('error', `Build failed: ${e}`)
      setResult({ success: false, error: String(e) })
    } finally {
      unlisten()
      setBuilding(false)
    }
  }

  const errors = validate()
  const canBuild = errors.length === 0 && !!projectPath

  return (
    <div>
      <div className="section-header">
        <div className="section-tag">05 — Build</div>
        <h1 className="section-title">Package installer</h1>
        <p className="section-desc">Bundle your files into a .lynxpak and generate the installer binary.</p>
      </div>

      {/* Checklist */}
      <div className="card">
        <div className="card-title">Pre-build checklist</div>
        {[
          { ok: !!projectPath,                    label: 'Project saved to disk',       fix: 'Use File → Save' },
          { ok: !!project.app.name.trim(),        label: 'App name set',                fix: 'Go to App Info' },
          { ok: !!project.app.version.trim(),     label: 'Version set',                 fix: 'Go to App Info' },
          { ok: !!project.app.id.trim(),          label: 'App ID set',                  fix: 'Go to App Info' },
          { ok: project.files.length > 0,         label: 'Files added',                 fix: 'Go to Files' },
          { ok: project.steps.length > 0,         label: 'Install steps defined',       fix: 'Go to Steps' },
        ].map((item, i) => (
          <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '6px 0', borderBottom: '1px solid var(--border)' }}>
            <span style={{ color: item.ok ? 'var(--success)' : 'var(--error)', fontSize: 13, width: 16 }}>
              {item.ok ? '✓' : '✗'}
            </span>
            <span style={{ flex: 1, fontSize: 12, color: item.ok ? 'var(--text)' : 'var(--text-muted)' }}>{item.label}</span>
            {!item.ok && <span style={{ fontSize: 11, color: 'var(--text-dim)' }}>{item.fix}</span>}
          </div>
        ))}
        <div style={{ marginTop: 16 }}>
          <button
            className={`btn btn-primary`}
            onClick={handleBuild}
            disabled={building || !canBuild}
            style={{ opacity: canBuild ? 1 : 0.4 }}
          >
            {building ? '⏳ Building...' : '▶ Build .lynxpak'}
          </button>
        </div>
      </div>

      {/* Build log */}
      {logs.length > 0 && (
        <div className="card">
          <div className="card-title">Build output</div>
          <div style={{
            background: 'var(--bg)',
            borderRadius: 4,
            padding: '12px 14px',
            fontFamily: 'JetBrains Mono, monospace',
            fontSize: 11,
            lineHeight: 1.7,
            maxHeight: 280,
            overflowY: 'auto',
          }}>
            {logs.map((log, i) => (
              <div key={i} style={{
                color: log.type === 'success' ? 'var(--success)'
                     : log.type === 'error'   ? 'var(--error)'
                     : log.type === 'progress' ? 'var(--text-dim)'
                     : 'var(--text-muted)',
              }}>
                <span style={{ color: 'var(--text-dim)', marginRight: 8 }}>{log.time}</span>
                {log.message}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Result */}
      {result && (
        <div className={`card`} style={{ borderColor: result.success ? 'rgba(61,220,132,0.2)' : 'rgba(255,92,92,0.2)' }}>
          <div className="card-title">{result.success ? '✅ Build successful' : '❌ Build failed'}</div>
          {result.success ? (
            <div style={{ fontSize: 12, color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>
              <div>Files packed: {result.file_count}</div>
              <div>Bundle size: {((result.total_bytes || 0) / 1024).toFixed(1)} KB</div>
              <div style={{ marginTop: 8, color: 'var(--accent)' }}>{result.output_path}</div>
            </div>
          ) : (
            <div style={{ fontSize: 12, color: 'var(--error)', fontFamily: 'JetBrains Mono, monospace' }}>{result.error}</div>
          )}
        </div>
      )}
    </div>
  )
}
