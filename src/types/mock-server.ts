// Mock Server-related types

/// CORS mode for the Mock Server
export type CorsMode = 'simple' | 'advanced';

/// Mock Server configuration
export interface MockServerConfig {
  id: number;
  port: number;
  corsMode: CorsMode;
  corsOrigins: string[] | null;
  corsMethods: string[] | null;
  corsHeaders: string[] | null;
  corsMaxAge: number;
  showDirectoryListing: boolean;
}

/// Directory mapping for the Mock Server
export interface DirectoryMapping {
  id: number;
  virtualPath: string;
  localPath: string;
  enabled: boolean;
}

/// Request to create a new directory mapping
export interface CreateMappingRequest {
  virtualPath: string;
  localPath: string;
}

/// Request to update a directory mapping
export interface UpdateMappingRequest {
  id: number;
  virtualPath: string | null;
  localPath: string | null;
  enabled: boolean | null;
}

/// Request to update Mock Server configuration
export interface UpdateConfigRequest {
  port: number | null;
  corsMode: CorsMode | null;
  corsOrigins: string[] | null;
  corsMethods: string[] | null;
  corsHeaders: string[] | null;
  corsMaxAge: number | null;
  showDirectoryListing: boolean | null;
}

/// Mock Server status
export type ServerStatus = 'running' | 'stopped';

/// Mock Server state information
export interface MockServerState {
  status: ServerStatus;
  port: number;
  url: string;
  mappingsCount: number;
}

/// Access log entry
export interface AccessLogEntry {
  timestamp: string;
  method: string;
  path: string;
  statusCode: number;
  responseSize: number | null;
  responseTimeMs: number;
}

/// File information for file browser
export interface FileInfo {
  name: string;
  path: string;
  isDirectory: boolean;
  size: number | null;
  mimeType: string | null;
}

