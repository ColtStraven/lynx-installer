import { useState } from 'react'
import type { LynxProject, FileEntry } from '../../App'

interface Props {
  project: LynxProject
  onChange: (updater: (p: LynxProject) => LynxProject) => void
}

const DEFAULT_ENTRY: FileEntry = {
  source: '',
  destination: '{install_dir}',
  recursive: true,
  filter: null,
  overwrite: true,
}

export default function FilesSection({ project, onChange }: Props) {
  const [editing, setEditing] = useState<number | null>(null)
  const [draft, setDraft] = useState<FileEntry>(DEFAULT_ENTRY)

  const addEntry = () => {
    setDraft(DEFAULT_ENTRY)
    setEditing(-1) // -1 = new
  }

  const saveEntry = () => {
    if (editing === -1) {
      onChange(p => ({ ...p, files: [...p.files, draft] }))
    } else if (editing !== null) {
      onChange(p => ({
        ...p,
        files: p.files.map((f, i) => i === editing ? draft : f)
      }))
    }
    setEditing(null)
  }

  const removeEntry = (i: number) => {
    onChange(p => ({ ...p, files: p.files.filter((_, idx) => idx !== i) }))
  }

  const editEntry = (i: number) => {
    setDraft({ ...project.files[i] })
    setEditing(i)
  }

  return (
    <div>
      <div className="section-header">
        <div className="section-tag">02 — Files</div>
        <h1 className="section-title">Files to bundle</h1>
        <p className="section-desc">Specify which files or directories to include in the installer.</p>
      </div>

      <div className="card">
        <div className="card-title" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span>File Entries</span>
          <button className="btn btn-sm btn-primary" onClick={addEntry}>+ Add Entry</button>
        </div>

        {project.files.length === 0 && editing === null ? (
          <div className="empty-state">
            <div className="empty-state-icon">⊞</div>
            <div className="empty-state-title">No files added yet</div>
            <div className="empty-state-desc">Add file entries to specify what gets installed.</div>
            <button className="btn btn-primary" onClick={addEntry}>Add first entry</button>
          </div>
        ) : (
          <>
            {project.files.map((entry, i) => (
              <div key={i} className="list-item">
                <div className="list-item-icon">📁</div>
                <div className="list-item-body">
                  <div className="list-item-title">{entry.source || '(no source)'}</div>
                  <div className="list-item-sub">→ {entry.destination}</div>
                </div>
                <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                  {entry.recursive && <span className="badge badge-orange">recursive</span>}
                  <div className="list-item-actions" style={{ opacity: 1 }}>
                    <button className="btn btn-icon btn-sm" onClick={() => editEntry(i)}>✎</button>
                    <button className="btn btn-icon btn-sm btn-danger" onClick={() => removeEntry(i)}>✕</button>
                  </div>
                </div>
              </div>
            ))}
          </>
        )}

        {editing !== null && (
          <div className="card" style={{ marginTop: 16, marginBottom: 0, background: 'var(--bg-2)' }}>
            <div className="card-title">{editing === -1 ? 'New File Entry' : 'Edit File Entry'}</div>
            <div className="field">
              <label>Source Path</label>
              <input
                type="text"
                value={draft.source}
                onChange={e => setDraft(d => ({ ...d, source: e.target.value }))}
                placeholder="dist/ or path/to/myapp.exe"
                autoFocus
              />
              <div className="token-hint">Relative to the .lynx project file</div>
            </div>
            <div className="field">
              <label>Destination</label>
              <input
                type="text"
                value={draft.destination}
                onChange={e => setDraft(d => ({ ...d, destination: e.target.value }))}
                placeholder="{install_dir}"
              />
              <div className="token-hint">
                Tokens: <span>{'{install_dir}'}</span> <span>{'{app_data}'}</span> <span>{'{temp}'}</span>
              </div>
            </div>
            <div className="field-row">
              <div className="field">
                <label>Filter (glob)</label>
                <input
                  type="text"
                  value={draft.filter ?? ''}
                  onChange={e => setDraft(d => ({ ...d, filter: e.target.value || null }))}
                  placeholder="*.exe (leave blank for all)"
                />
              </div>
              <div className="field" style={{ display: 'flex', flexDirection: 'column', justifyContent: 'flex-end', gap: 8 }}>
                <label>
                  <input type="checkbox" checked={draft.recursive} onChange={e => setDraft(d => ({ ...d, recursive: e.target.checked }))} style={{ marginRight: 6 }} />
                  Include subdirectories
                </label>
                <label>
                  <input type="checkbox" checked={draft.overwrite} onChange={e => setDraft(d => ({ ...d, overwrite: e.target.checked }))} style={{ marginRight: 6 }} />
                  Overwrite existing files
                </label>
              </div>
            </div>
            <div style={{ display: 'flex', gap: 8, marginTop: 4 }}>
              <button className="btn btn-primary" onClick={saveEntry}>Save Entry</button>
              <button className="btn" onClick={() => setEditing(null)}>Cancel</button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
