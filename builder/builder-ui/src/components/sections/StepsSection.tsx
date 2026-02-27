import type { LynxProject, InstallStep } from '../../App'

interface Props {
  project: LynxProject
  onChange: (updater: (p: LynxProject) => LynxProject) => void
}

const STEP_ICONS: Record<string, string> = {
  extract:             '📦',
  shortcut:            '🔗',
  registry:            '🗝',
  command:             '⚡',
  env_var:             '🌐',
  register_uninstaller:'🛡',
}

const STEP_LABELS: Record<string, string> = {
  extract:             'Extract Files',
  shortcut:            'Create Shortcut',
  registry:            'Registry Entry',
  command:             'Run Command',
  env_var:             'Environment Variable',
  register_uninstaller:'Register Uninstaller',
}

function stepSummary(step: InstallStep): string {
  switch (step.kind) {
    case 'shortcut': return `${step.locations?.join(', ') || 'desktop'}`
    case 'registry': return `${step.hive}\\${step.key}`
    case 'command':  return step.command
    case 'env_var':  return `${(step as any).name} = ${(step as any).value}`
    default:         return step.label
  }
}

export default function StepsSection({ project, onChange }: Props) {
  const addStep = (kind: InstallStep['kind']) => {
    const defaults: Record<string, Partial<InstallStep>> = {
      extract:             { kind: 'extract', label: 'Installing application files...' },
      shortcut:            { kind: 'shortcut', label: 'Creating shortcuts...', target: '{install_dir}/{app_name}.exe', locations: ['desktop', 'start_menu'], name: null } as any,
      registry:            { kind: 'registry', label: 'Writing registry...', hive: 'HKEY_CURRENT_USER', key: 'SOFTWARE\\MyApp', value_name: 'InstallPath', value_data: '{install_dir}', value_type: 'REG_SZ' } as any,
      command:             { kind: 'command', label: 'Running command...', command: '', args: [], wait: true, fail_on_error: false } as any,
      register_uninstaller:{ kind: 'register_uninstaller', label: 'Finalizing installation...' },
    }
    const newStep = defaults[kind] as InstallStep
    onChange(p => ({ ...p, steps: [...p.steps, newStep] }))
  }

  const removeStep = (i: number) =>
    onChange(p => ({ ...p, steps: p.steps.filter((_, idx) => idx !== i) }))

  const moveUp = (i: number) => {
    if (i === 0) return
    onChange(p => {
      const steps = [...p.steps]
      ;[steps[i - 1], steps[i]] = [steps[i], steps[i - 1]]
      return { ...p, steps }
    })
  }

  const moveDown = (i: number) => {
    onChange(p => {
      if (i >= p.steps.length - 1) return p
      const steps = [...p.steps]
      ;[steps[i], steps[i + 1]] = [steps[i + 1], steps[i]]
      return { ...p, steps }
    })
  }

  const updateLabel = (i: number, label: string) =>
    onChange(p => ({
      ...p,
      steps: p.steps.map((s, idx) => idx === i ? { ...s, label } : s)
    }))

  return (
    <div>
      <div className="section-header">
        <div className="section-tag">03 — Steps</div>
        <h1 className="section-title">Install steps</h1>
        <p className="section-desc">Define what happens when the user clicks Install. Steps run in order.</p>
      </div>

      <div className="card">
        <div className="card-title" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span>Steps ({project.steps.length})</span>
          <div style={{ display: 'flex', gap: 6 }}>
            {(['extract', 'shortcut', 'registry', 'command', 'register_uninstaller'] as InstallStep['kind'][]).map(kind => (
              <button key={kind} className="btn btn-sm" onClick={() => addStep(kind)}>
                + {STEP_LABELS[kind]}
              </button>
            ))}
          </div>
        </div>

        {project.steps.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">◎</div>
            <div className="empty-state-title">No steps defined</div>
            <div className="empty-state-desc">Add at least an Extract step to install files.</div>
            <button className="btn btn-primary" onClick={() => addStep('extract')}>Add Extract step</button>
          </div>
        ) : (
          project.steps.map((step, i) => (
            <div key={i} className="list-item" style={{ alignItems: 'flex-start' }}>
              <div className="list-item-icon" style={{ marginTop: 2 }}>
                {STEP_ICONS[step.kind] || '◎'}
              </div>
              <div className="list-item-body">
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                  <span className="badge badge-orange">{step.kind}</span>
                </div>
                <input
                  type="text"
                  value={step.label}
                  onChange={e => updateLabel(i, e.target.value)}
                  style={{ marginBottom: 4 }}
                  placeholder="Step label shown in installer..."
                />
                <div className="list-item-sub">{stepSummary(step)}</div>
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 3, marginLeft: 4 }}>
                <button className="btn btn-icon btn-sm" onClick={() => moveUp(i)} title="Move up">↑</button>
                <button className="btn btn-icon btn-sm" onClick={() => moveDown(i)} title="Move down">↓</button>
                <button className="btn btn-icon btn-sm btn-danger" onClick={() => removeStep(i)} title="Remove">✕</button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  )
}
