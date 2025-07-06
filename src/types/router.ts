// 路由ID类型定义
export type RouteId = 'timing' | 'accounting' | 'notes' | 'data' | 'settings' | 'about' | 'system' | 'data-export' | 'data-import' | 'data-backup' | 'data-sync' | 'data-cleanup';

// 导航来源类型
export type NavigationSource = 'direct' | 'system';

// 路由记录类型
export interface RouteRecord {
  route: RouteId;
  source: NavigationSource;
  timestamp: number;
}

// 路由状态类型
export interface RouteState {
  current: RouteId;
  source: NavigationSource;
  stack: RouteRecord[];
  canGoBack: boolean;
}

// 路由配置类型
export interface RouterConfig {
  rememberNavigation: boolean;
  defaultRoute: RouteId;
}

// 路由操作类型
export interface RouterActions {
  navigate: (route: RouteId, source?: NavigationSource) => void;
  goBack: () => void;
  reset: () => void;
}

// 路由上下文类型
export interface RouterContext {
  state: RouteState;
  actions: RouterActions;
  config: RouterConfig;
  updateConfig: (newConfig: Partial<RouterConfig>) => void;
}

// 路由历史类型
export interface RouteHistory {
  records: RouteRecord[];
  maxSize: number;
} 