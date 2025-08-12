export type BackendKind = 'wsl' | 'docker'

export interface DistroVersion {
  version: string
  channel?: string
}

export interface Distro {
  id: string
  name: string
  vendor?: string
  website?: string
  backends: BackendKind[]
}