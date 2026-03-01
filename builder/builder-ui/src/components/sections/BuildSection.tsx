import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { saveExeFile, pickDirectory } from '../../dialogs'
import type { LynxProject } from '../../App'

interface Props {
  project: LynxProject
  projectPath: string | null
  onSave: () => Promise<string | null>
}

interface BuildLog {
  type: 'info' | 'success' | 'error'
  message: string
  time: string
}

interface Stage {
  current: number
  total: number
  label: string
}

const STAGE_ICONS: Record<string, string> = {
  'Loading project':           '📂',
  'Bundling files':            '📦',
  'Preparing build directory': '🗂',
  'Building frontend':         '⚙',
  'Patching shell config':     '🔧',
  'Building uninstaller':      '🛡',
  'Compiling release binary':  '🔨',
  'Copying output':            '📋',
}

export default function BuildSection({ project, projectPath, onSave }: Props) {
  const [building, setBuilding]   = useState(false)
  const [logs, setLogs]           = useState<BuildLog[]>([])
  const [outputPath, setOutputPath] = useState('')
  const [shellPath, setShellPath] = useState('')
  const [stage, setStage]         = useState<Stage | null>(null)
  const [result, setResult]       = useState<null | { success: boolean; output_path?: string; size_bytes?: number; error?: string }>(null)

  const addLog = (type: BuildLog['type'], message: string) => {
    const time = new Date().toLocaleTimeString('en', { hour12: false })
    setLogs(prev => [...prev, { type, message, time }])
  }

  const validate = () => {
    const errors: string[] = []
    if (!project.app.name.trim())      errors.push('App name is required')
    if (!project.app.version.trim())   errors.push('Version is required')
    if (!project.app.id.trim())        errors.push('App ID is required')
    if (project.files.length === 0)    errors.push('No files added — nothing to install')
    if (project.steps.length === 0)    errors.push('No install steps defined')
    if (!shellPath.trim())             errors.push('Shell path is required')
    return errors
  }

  const pickShellDir = async () => {
    const path = await pickDirectory()
    if (path) setShellPath(path)
  }

  const pickOutputPath = async () => {
    const safeName = project.app.name.replace(/\s+/g, '-')
    const path = await saveExeFile(`${safeName}-installer.exe`)
    if (path) setOutputPath(path)
  }

  const handleBuild = async () => {
    setBuilding(true)
    setLogs([])
    setResult(null)
    setStage(null)

    addLog('info', 'Saving project...')
    const savedPath = await onSave()
    if (!savedPath) {
      addLog('error', 'Build cancelled — project must be saved first')
      setBuilding(false)
      return
    }

    const errors = validate()
    if (errors.length > 0) {
      errors.forEach(e => addLog('error', e))
      setBuilding(false)
      return
    }

    const finalOutput = outputPath || (savedPath.replace(/\.lynx$/, '') + '-installer.exe')

    addLog('info', `Building ${project.app.name} v${project.app.version}...`)

    const unlisten = await listen<{ type: string; message: string; stage?: Stage }>('build-progress', event => {
      const { type, message, stage: s } = event.payload
      if (s) setStage(s)
      addLog(type === 'error' ? 'error' : 'info', message)
      setTimeout(() => {
        const log = document.getElementById('build-log')
        if (log) log.scrollTop = log.scrollHeight
      }, 50)
    })

    try {
      const res = await invoke<typeof result>('build_installer', {
        projectPath: savedPath,
        shellPath,
        outputPath: finalOutput,
      })
      setResult(res)
      if (res?.success) {
        const kb = ((res.size_bytes || 0) / 1024).toFixed(0)
        addLog('success', `✅ Build complete! ${kb} KB → ${res.output_path}`)
        setStage({ current: 8, total: 8, label: 'Done' })
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
  const canBuild = errors.length === 0
  const progressPct = stage ? Math.round((stage.current / stage.total) * 100) : 0

  return (
    <div>
      <div className="section-header">
        <div className="section-tag">05 — Build</div>
        <h1 className="section-title">Build installer</h1>
        <p className="section-desc">Compile your project into a single redistributable .exe file.</p>
      </div>

      {/* Config */}
      <div className="card">
        <div className="card-title">Build configuration</div>
        <div className="field">
          <label>Shell directory *</label>
          <div style={{ display: 'flex', gap: 8 }}>
            <input
              type="text"
              value={shellPath}
              onChange={e => setShellPath(e.target.value)}
              placeholder="Path to the shell/ directory"
              style={{ flex: 1 }}
            />
            <button className="btn btn-sm" onClick={pickShellDir}>Browse</button>
          </div>
          <div className="token-hint">The shell/ folder in your lynx-installer repo</div>
        </div>
        <div className="field">
          <label>Output path</label>
          <div style={{ display: 'flex', gap: 8 }}>
            <input
              type="text"
              value={outputPath}
              onChange={e => setOutputPath(e.target.value)}
              placeholder={projectPath ? projectPath.replace(/\.lynx$/, '-installer.exe') : 'MyApp-installer.exe'}
              style={{ flex: 1 }}
            />
            <button className="btn btn-sm" onClick={pickOutputPath}>Browse</button>
          </div>
          <div className="token-hint">Leave blank to save next to the .lynx project file</div>
        </div>
      </div>

      {/* Checklist */}
      <div className="card">
        <div className="card-title">Pre-build checklist</div>
        {[
          { ok: !!project.app.name.trim(),     label: 'App name set',             fix: 'App Info section' },
          { ok: !!project.app.version.trim(),  label: 'Version set',              fix: 'App Info section' },
          { ok: !!project.app.id.trim(),       label: 'App ID set',               fix: 'App Info section' },
          { ok: project.files.length > 0,      label: 'Files added',              fix: 'Files section' },
          { ok: project.steps.length > 0,      label: 'Install steps defined',    fix: 'Steps section' },
          { ok: !!shellPath.trim(),             label: 'Shell path set',           fix: 'Set above' },
          { ok: !!project.app.icon, optional: true, label: 'App icon (optional)', fix: 'App Info section' },
        ].map((item, i) => (
          <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '6px 0', borderBottom: '1px solid var(--border)' }}>
            <span style={{
              color: item.ok ? 'var(--success)'
                   : ('optional' in item && item.optional) ? 'var(--text-dim)'
                   : 'var(--error)',
              fontSize: 13, width: 16, flexShrink: 0
            }}>
              {item.ok ? '✓' : '✗'}
            </span>
            <span style={{ flex: 1, fontSize: 12, color: item.ok ? 'var(--text)' : 'var(--text-muted)' }}>{item.label}</span>
            {!item.ok && <span style={{ fontSize: 11, color: 'var(--text-dim)' }}>{item.fix}</span>}
          </div>
        ))}

        <div style={{ marginTop: 16, display: 'flex', alignItems: 'center', gap: 12 }}>
          <button
            className="btn btn-primary"
            onClick={handleBuild}
            disabled={building}
            style={{ opacity: canBuild ? 1 : 0.5 }}
          >
            {building ? '⏳ Building...' : '▶ Build Installer'}
          </button>
          {!canBuild && !building && (
            <span style={{ fontSize: 11, color: 'var(--text-dim)' }}>
              Fix {errors.length} issue{errors.length !== 1 ? 's' : ''} above
            </span>
          )}
        </div>
      </div>

      {/* Progress bar — shown while building or after completion */}
      {(building || stage) && (
        <div className="card">
          <div className="card-title" style={{ marginBottom: 12 }}>
            {building ? 'Build progress' : result?.success ? '✅ Build complete' : '❌ Build failed'}
          </div>

          {/* Stage label */}
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 6, fontSize: 12 }}>
            <span style={{ color: 'var(--text)', fontWeight: 500 }}>
              {stage ? `${STAGE_ICONS[stage.label] ?? '⚙'} ${stage.label}` : 'Starting...'}
            </span>
            <span style={{ color: 'var(--text-dim)' }}>
              {stage ? `${stage.current} / ${stage.total}` : ''}
            </span>
          </div>

          {/* Progress bar */}
          <div style={{
            height: 6,
            background: 'var(--bg)',
            borderRadius: 3,
            overflow: 'hidden',
            marginBottom: 14,
          }}>
            <div style={{
              height: '100%',
              width: `${progressPct}%`,
              background: result?.success
                ? 'var(--success)'
                : result && !result.success
                  ? 'var(--error)'
                  : 'var(--accent)',
              borderRadius: 3,
              transition: 'width 0.4s ease',
            }} />
          </div>

          {/* Stage steps row */}
          <div style={{ display: 'flex', gap: 4 }}>
            {Array.from({ length: 8 }, (_, i) => {
              const done  = stage ? i < stage.current : false
              const active = stage ? i === stage.current - 1 : false
              return (
                <div
                  key={i}
                  style={{
                    flex: 1,
                    height: 3,
                    borderRadius: 2,
                    background: done
                      ? result?.success ? 'var(--success)' : 'var(--accent)'
                      : active
                        ? 'var(--accent)'
                        : 'var(--border)',
                    opacity: active ? 1 : done ? 0.7 : 0.3,
                    transition: 'background 0.3s, opacity 0.3s',
                  }}
                />
              )
            })}
          </div>

          {/* Compiling note */}
          {building && stage?.label === 'Compiling release binary' && (
            <div style={{ marginTop: 10, fontSize: 11, color: 'var(--text-dim)', fontStyle: 'italic' }}>
              ☕ Compiling Rust — this takes 2–4 minutes on first build, ~30s with cache
            </div>
          )}
        </div>
      )}

      {/* Build log */}
      {logs.length > 0 && (
        <div className="card">
          <div className="card-title">Build output</div>
          <div
            id="build-log"
            style={{
              background: 'var(--bg)',
              borderRadius: 4,
              padding: '12px 14px',
              fontFamily: 'JetBrains Mono, monospace',
              fontSize: 11,
              lineHeight: 1.7,
              maxHeight: 260,
              overflowY: 'auto',
            }}
          >
            {logs.map((log, i) => (
              <div key={i} style={{
                color: log.type === 'success' ? 'var(--success)'
                     : log.type === 'error'   ? 'var(--error)'
                     : 'var(--text-muted)',
              }}>
                <span style={{ color: 'var(--text-dim)', marginRight: 8 }}>{log.time}</span>
                {log.message}
              </div>
            ))}
            {building && (
              <div style={{ color: 'var(--accent)', marginTop: 4 }}>▌</div>
            )}
          </div>
        </div>
      )}

      {/* Result */}
      {result && (
        <div className="card" style={{ borderColor: result.success ? 'rgba(61,220,132,0.2)' : 'rgba(255,92,92,0.2)' }}>
          <div className="card-title">{result.success ? '✅ Build successful' : '❌ Build failed'}</div>
          {result.success ? (
            <div style={{ fontSize: 12, color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>
              <div>Size: {((result.size_bytes || 0) / 1024).toFixed(0)} KB</div>
              <div style={{ marginTop: 6, color: 'var(--accent)' }}>{result.output_path}</div>
            </div>
          ) : (
            <div style={{ fontSize: 12, color: 'var(--error)', fontFamily: 'JetBrains Mono, monospace' }}>{result.error}</div>
          )}
        </div>
      )}
    </div>
  )
}