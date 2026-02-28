// Native dialog helpers using tauri-plugin-dialog JS API
import { save, open } from '@tauri-apps/plugin-dialog'

export async function saveLynxFile(defaultName: string): Promise<string | null> {
  console.log('[dialogs] saveLynxFile called, defaultName:', defaultName)
  try {
    const result = await save({
      defaultPath: defaultName,
      filters: [{ name: 'Lynx Project', extensions: ['lynx'] }],
    })
    console.log('[dialogs] saveLynxFile result:', result, typeof result)
    return result ?? null
  } catch (e) {
    console.error('[dialogs] saveLynxFile error:', e)
    return null
  }
}

export async function saveExeFile(defaultName: string): Promise<string | null> {
  try {
    const result = await save({
      defaultPath: defaultName,
      filters: [{ name: 'Executable', extensions: ['exe'] }],
    })
    console.log('[dialogs] saveExeFile result:', result)
    return result ?? null
  } catch (e) {
    console.error('[dialogs] saveExeFile error:', e)
    return null
  }
}

export async function pickLynxFile(): Promise<string | null> {
  try {
    const result = await open({
      multiple: false,
      filters: [{ name: 'Lynx Project', extensions: ['lynx'] }],
    })
    console.log('[dialogs] pickLynxFile result:', result)
    return (result as string | null) ?? null
  } catch (e) {
    console.error('[dialogs] pickLynxFile error:', e)
    return null
  }
}

export async function pickDirectory(): Promise<string | null> {
  try {
    const result = await open({
      directory: true,
      multiple: false,
    })
    console.log('[dialogs] pickDirectory result:', result)
    return (result as string | null) ?? null
  } catch (e) {
    console.error('[dialogs] pickDirectory error:', e)
    return null
  }
}