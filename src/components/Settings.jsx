import React, { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  AlertTriangle,
  CheckCircle2,
  FolderOpen,
  HardDrive,
  Info,
  RotateCcw,
  Save,
  Settings as SettingsIcon,
  XCircle,
} from 'lucide-react'
import { CustomSelect, Spinner } from './UI.jsx'
import * as api from '../api.js'

function SourceBadge({ source }) {
  if (source === 'custom') {
    return <span className="badge badge-ok">Manual override</span>
  }
  if (source === 'detected') {
    return <span className="badge badge-ok">Auto-detected</span>
  }
  if (source && source.startsWith('env:')) {
    return <span className="badge badge-ok">Env: {source.slice(4)}</span>
  }
  return <span className="badge badge-gpu">Managed</span>
}

function ComponentDot({ ok, label }) {
  return (
    <span
      className="flex items-center gap-1"
      style={{ fontSize: 11, color: ok ? 'var(--text-green)' : 'var(--text-muted)' }}
    >
      {ok ? <CheckCircle2 size={12} /> : <XCircle size={12} style={{ opacity: 0.5 }} />}
      {label}
    </span>
  )
}

export function Settings({
  toast,
  refreshStatus,
}) {
  const { t, i18n } = useTranslation()
  const currentLang = i18n.language || 'en'
  const [draftLang, setDraftLang] = useState(currentLang)

  // ── Manual SDK path state ──────────────────────────────────────────────
  const [sdkInfo, setSdkInfo] = useState(null)
  const [sdkLoading, setSdkLoading] = useState(true)
  const [sdkLoadError, setSdkLoadError] = useState(null)
  const [sdkDraft, setSdkDraft] = useState('')
  const [sdkValidation, setSdkValidation] = useState(null)
  const [sdkValidating, setSdkValidating] = useState(false)
  const [sdkSaving, setSdkSaving] = useState(false)
  const [sdkClearing, setSdkClearing] = useState(false)
  const [browsing, setBrowsing] = useState(false)

  const languages = Object.keys(i18n.store?.data || {}).map(langCode => {
    const translation = i18n.store.data[langCode]?.translation || {}
    return {
      code: langCode,
      name: translation.lang_name || langCode.toUpperCase()
    }
  })

  const handleSave = () => {
    i18n.changeLanguage(draftLang)
    localStorage.setItem('app_lang', draftLang)
    toast(t('ready') + '! Settings updated.', 'info')
  }

  // ── SDK path helpers ───────────────────────────────────────────────────
  const loadSdkInfo = async () => {
    setSdkLoading(true)
    setSdkLoadError(null)
    try {
      const info = await api.getSdkPathInfo()
      setSdkInfo(info)
      // Pre-fill the editor with the override (if any), else the active path.
      setSdkDraft(info.custom_path || info.current_path || '')
      setSdkValidation(null)
    } catch (e) {
      console.error('getSdkPathInfo failed:', e)
      setSdkLoadError(String(e))
    }
    setSdkLoading(false)
  }

  useEffect(() => {
    loadSdkInfo()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const afterSdkChange = async () => {
    await loadSdkInfo()
    if (typeof refreshStatus === 'function') {
      try { await refreshStatus() } catch { /* non-fatal */ }
    }
  }

  const handleBrowse = async () => {
    setBrowsing(true)
    try {
      // Lazy import so plain `vite dev` (browser) doesn't crash without Tauri.
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        directory: true,
        multiple: false,
        recursive: false,
        title: 'Select Android SDK folder',
      })
      if (typeof selected === 'string' && selected) {
        setSdkDraft(selected)
        // Validate immediately so the user gets instant feedback.
        await handleValidate(selected)
      }
    } catch (e) {
      console.error('Browse dialog failed:', e)
      toast('Folder browser is only available in the desktop app. Paste the path manually.', 'warn')
    }
    setBrowsing(false)
  }

  const handleValidate = async (pathOverride) => {
    const target = (pathOverride ?? sdkDraft ?? '').trim()
    if (!target) {
      setSdkValidation(null)
      return
    }
    setSdkValidating(true)
    try {
      const v = await api.validateSdkPath({ path: target })
      setSdkValidation(v)
    } catch (e) {
      setSdkValidation({ valid: false, message: String(e), path: target })
    }
    setSdkValidating(false)
  }

  const handleSaveSdk = async () => {
    const target = (sdkDraft || '').trim()
    if (!target) {
      toast('Enter or browse to an SDK folder first.', 'warn')
      return
    }
    setSdkSaving(true)
    try {
      const r = await api.setCustomSdkPath({ path: target })
      if (r && r.ok) {
        toast('Manual SDK path saved!', 'success')
        await afterSdkChange()
      } else {
        toast(`Invalid SDK folder: ${r?.error || 'unknown error'}`, 'error')
        await handleValidate(target)
      }
    } catch (e) {
      toast(`Failed to save SDK path: ${e}`, 'error')
    }
    setSdkSaving(false)
  }

  const handleClearSdk = async () => {
    setSdkClearing(true)
    try {
      const r = await api.clearCustomSdkPath()
      if (r && r.ok) {
        toast('Manual override cleared — back to auto-detect.', 'success')
        await afterSdkChange()
      } else {
        toast(`Failed to reset: ${r?.error || 'unknown error'}`, 'error')
      }
    } catch (e) {
      toast(`Failed to reset SDK path: ${e}`, 'error')
    }
    setSdkClearing(false)
  }

  const handleUseCandidate = async (path) => {
    setSdkDraft(path)
    setSdkSaving(true)
    try {
      const r = await api.setCustomSdkPath({ path })
      if (r && r.ok) {
        toast('Manual SDK path saved!', 'success')
        await afterSdkChange()
      } else {
        toast(`Invalid SDK folder: ${r?.error || 'unknown error'}`, 'error')
      }
    } catch (e) {
      toast(`Failed to save SDK path: ${e}`, 'error')
    }
    setSdkSaving(false)
  }

  return (
    <div className="page fade-in">
      <div className="page-header">
        <h1 className="page-title flex items-center gap-2">
          <SettingsIcon size={20} />
          {t('settings_title')}
        </h1>
        <p className="page-subtitle">{t('settings_subtitle')}</p>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: 720 }}>
        {/* ── Interface ── */}
        <div className="card">
          <div className="card-title">{t('settings_section_interface')}</div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16, marginTop: 12 }}>
            <div className="form-group">
              <label className="form-label">{t('settings_lang_label')}</label>
              <CustomSelect
                value={draftLang}
                onChange={setDraftLang}
                options={languages.map(l => ({ value: l.code, label: l.name }))}
              />
            </div>
          </div>
        </div>

        <button
          className="btn btn-primary"
          onClick={handleSave}
          style={{ alignSelf: 'flex-start', marginTop: 8 }}
        >
          {t('settings_save_btn')}
        </button>

        {/* ── Manual SDK path (fallback for non-standard installs) ── */}
        <div className="card" id="sdk-path-card">
          <div className="card-title">
            <HardDrive size={14} style={{ marginRight: 6 }} />
            {t('settings_section_sdk', 'Android SDK Location')}
          </div>
          <div style={{ fontSize: 12, color: 'var(--text-secondary)', marginBottom: 14, lineHeight: 1.5 }}>
            {t(
              'settings_sdk_desc',
              'The app auto-detects your Android SDK. If yours lives in a non-standard folder, browse and set it here as a manual fallback so emulator, ADB and system-image tools keep working.'
            )}
          </div>

          {sdkLoading ? (
            <div className="flex items-center gap-2" style={{ padding: '12px 0' }}>
              <Spinner size={14} />
              <span className="text-muted" style={{ fontSize: 12 }}>Resolving SDK location…</span>
            </div>
          ) : sdkLoadError ? (
            <div className="alert alert-warn">
              <AlertTriangle size={14} style={{ flexShrink: 0, marginTop: 1 }} />
              <span>Could not load SDK info ({sdkLoadError}). Are you running inside the desktop app?</span>
            </div>
          ) : sdkInfo && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
              {/* Active path summary */}
              <div
                style={{
                  background: 'rgba(255,255,255,0.02)',
                  border: '1px solid var(--border)',
                  borderRadius: 10,
                  padding: '10px 12px',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 8,
                }}
              >
                <div className="flex items-center gap-2" style={{ flexWrap: 'wrap' }}>
                  <span style={{ fontSize: 11, fontWeight: 600, color: 'var(--text-secondary)' }}>
                    {t('settings_sdk_current', 'Active SDK path')}
                  </span>
                  <SourceBadge source={sdkInfo.source} />
                  {sdkInfo.valid
                    ? <span className="badge badge-ok"><CheckCircle2 size={10} /> Valid</span>
                    : <span className="badge badge-warn"><AlertTriangle size={10} /> Missing components</span>
                  }
                </div>
                <div
                  className="font-mono"
                  style={{ fontSize: 11.5, wordBreak: 'break-all', color: 'var(--text-primary)' }}
                  title={sdkInfo.current_path}
                >
                  {sdkInfo.current_path || '(not resolved)'}
                </div>
                <div className="flex items-center gap-2" style={{ flexWrap: 'wrap', gap: 12 }}>
                  <ComponentDot ok={sdkInfo.has_emulator} label="emulator" />
                  <ComponentDot ok={sdkInfo.has_platform_tools} label="platform-tools" />
                  <ComponentDot ok={sdkInfo.has_cmdline_tools} label="cmdline-tools" />
                  <ComponentDot ok={sdkInfo.has_system_images} label="system-images" />
                </div>
                {sdkInfo.message && (
                  <div style={{ fontSize: 11, color: 'var(--text-muted)', lineHeight: 1.45 }}>
                    {sdkInfo.message}
                  </div>
                )}
              </div>

              {/* Editor */}
              <div className="form-group">
                <label className="form-label">{t('settings_sdk_input_label', 'SDK folder path (manual override)')}</label>
                <div className="flex gap-2" style={{ alignItems: 'stretch' }}>
                  <input
                    id="sdk-path-input"
                    className="form-input font-mono"
                    value={sdkDraft}
                    onChange={e => {
                      setSdkDraft(e.target.value)
                      setSdkValidation(null)
                    }}
                    onBlur={() => handleValidate()}
                    placeholder="e.g. C:\Android\Sdk or D:\Dev\Android\Sdk"
                    style={{ flex: 1, borderRadius: 10 }}
                  />
                  <button
                    id="btn-sdk-browse"
                    className="btn btn-ghost btn-sm"
                    onClick={handleBrowse}
                    disabled={browsing}
                    title="Browse for your Android SDK folder"
                    style={{ whiteSpace: 'nowrap' }}
                  >
                    {browsing ? <Spinner size={12} /> : <FolderOpen size={12} style={{ marginRight: 6 }} />}
                    {t('settings_sdk_browse', 'Browse…')}
                  </button>
                </div>
                <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 6, lineHeight: 1.5 }}>
                  Pick the folder that directly contains <span className="font-mono">emulator</span>,{' '}
                  <span className="font-mono">platform-tools</span> or{' '}
                  <span className="font-mono">cmdline-tools</span> — not a subfolder.
                </div>
              </div>

              {/* Validation feedback */}
              {(sdkValidating || sdkValidation) && (
                <div
                  className={`alert ${sdkValidation?.valid ? 'alert-success' : 'alert-warn'}`}
                  style={{ fontSize: 12 }}
                >
                  {sdkValidating ? (
                    <span className="flex items-center gap-2"><Spinner size={12} /> Validating…</span>
                  ) : sdkValidation?.valid ? (
                    <span className="flex items-center gap-2">
                      <CheckCircle2 size={14} style={{ flexShrink: 0 }} />
                      {sdkValidation.message}
                    </span>
                  ) : (
                    <span className="flex items-center gap-2">
                      <AlertTriangle size={14} style={{ flexShrink: 0 }} />
                      {sdkValidation?.message || 'Invalid path.'}
                    </span>
                  )}
                </div>
              )}

              {/* Actions */}
              <div className="flex gap-2" style={{ flexWrap: 'wrap' }}>
                <button
                  id="btn-sdk-save"
                  className="btn btn-primary btn-sm"
                  onClick={handleSaveSdk}
                  disabled={sdkSaving || sdkValidating}
                >
                  {sdkSaving ? <Spinner size={12} /> : <Save size={12} style={{ marginRight: 6 }} />}
                  {t('settings_sdk_save', 'Use This SDK')}
                </button>
                <button
                  id="btn-sdk-validate"
                  className="btn btn-ghost btn-sm"
                  onClick={() => handleValidate()}
                  disabled={sdkValidating || !(sdkDraft || '').trim()}
                >
                  <Info size={12} style={{ marginRight: 6 }} />
                  Validate
                </button>
                {sdkInfo.custom_path && (
                  <button
                    id="btn-sdk-reset"
                    className="btn btn-ghost btn-sm"
                    onClick={handleClearSdk}
                    disabled={sdkClearing}
                    title="Remove the manual override and go back to auto-detect"
                  >
                    {sdkClearing ? <Spinner size={12} /> : <RotateCcw size={12} style={{ marginRight: 6 }} />}
                    {t('settings_sdk_reset', 'Reset to Auto')}
                  </button>
                )}
              </div>

              {/* Detected candidates */}
              {sdkInfo.detected_candidates && sdkInfo.detected_candidates.length > 0 && (
                <div>
                  <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--text-secondary)', marginBottom: 8 }}>
                    {t('settings_sdk_detected', 'Detected SDKs on this PC')}
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                    {sdkInfo.detected_candidates.map(p => {
                      const isActive = p === sdkInfo.current_path
                      return (
                        <div key={p} className="pkg-row" style={{ padding: '6px 10px' }}>
                          <span className="font-mono" style={{ fontSize: 11, flex: 1, wordBreak: 'break-all' }}>{p}</span>
                          {isActive
                            ? <span className="badge badge-ok">Active</span>
                            : (
                              <button
                                className="btn btn-ghost btn-sm"
                                style={{ fontSize: 10, padding: '3px 10px' }}
                                onClick={() => handleUseCandidate(p)}
                                disabled={sdkSaving}
                              >
                                {t('settings_sdk_use', 'Use')}
                              </button>
                            )
                          }
                        </div>
                      )
                    })}
                  </div>
                </div>
              )}

              {/* Managed path reference */}
              <div style={{ fontSize: 11, color: 'var(--text-muted)', lineHeight: 1.5 }}>
                {t('settings_sdk_managed', 'App-managed SDK')}:{' '}
                <span className="font-mono">{sdkInfo.managed_path}</span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
