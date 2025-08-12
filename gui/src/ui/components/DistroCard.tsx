import React, { useMemo, useState } from 'react'
import { motion } from 'framer-motion'
import { Distro, DistroVersion } from '../types'

export function DistroCard({ distro, onSelect, versions, selected, onInstall }: {
  distro: Distro
  onSelect: () => void
  versions: DistroVersion[]
  selected: boolean
  onInstall: (version: string, backend: 'wsl' | 'docker') => Promise<void>
}) {
  const [ver, setVer] = useState('')
  const icon = useMemo(() => `/icons/${distro.id}.svg`, [distro.id])

  return (
    <motion.div className={`card ${selected ? 'selected' : ''}`} whileHover={{ scale: 1.02 }} onClick={onSelect}>
      <img src={icon} alt={`${distro.name} icon`} className="icon" onError={(e) => { (e.target as HTMLImageElement).src = '/icons/linux.svg' }} />
      <div className="title">{distro.name}</div>
      {selected && (
        <div className="actions" onClick={e => e.stopPropagation()}>
          <select value={ver} onChange={e => setVer(e.target.value)}>
            <option value="">Select version…</option>
            {versions.map(v => <option key={v.version} value={v.version}>{v.version}{v.channel ? ` (${v.channel})` : ''}</option>)}
          </select>
          <div className="btn-row">
            <button disabled={!ver} onClick={() => ver && onInstall(ver, 'wsl')}>Install (WSL)</button>
            <button disabled={!ver} onClick={() => ver && onInstall(ver, 'docker')}>Install (Docker)</button>
          </div>
        </div>
      )}
    </motion.div>
  )
}