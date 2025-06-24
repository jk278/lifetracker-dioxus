// 任务数据类型
export interface Task {
  id: string;
  name: string;
  description?: string;
  category_id?: string;
  category_name?: string;
  start_time?: string;
  end_time?: string;
  duration_seconds: number;
  is_active: boolean;
  tags: string[];
  created_at: string;
  updated_at: string;
}

// 分类数据类型
export interface Category {
  id: string;
  name: string;
  description?: string;
  color: string;
  icon?: string;
  is_active: boolean;
  task_count: number;
  total_duration_seconds: number;
  created_at: string;
  updated_at: string;
}

// 计时器状态类型
export interface TimerStatus {
  state: 'running' | 'paused' | 'stopped';
  current_task_id?: string;
  current_task_name?: string;
  start_time?: string;
  pause_time?: string;
  elapsed_seconds: number;
  total_today_seconds: number;
}

// 时间记录类型
export interface TimeEntry {
  id: number;
  task_name: string;
  category_name?: string;
  start_time: string;
  end_time?: string;
  duration_seconds: number;
}

// 统计数据类型
export interface Statistics {
  today: {
    total_seconds: number;
    task_count: number;
    active_categories: number;
    most_productive_hour?: number;
  };
  this_week: {
    total_seconds: number;
    task_count: number;
    active_days: number;
    average_daily_seconds: number;
  };
  category_stats: {
    category_id: string;
    category_name: string;
    total_seconds: number;
    task_count: number;
    percentage: number;
  }[];
}

// 创建任务请求类型
export interface CreateTaskRequest {
  name: string;
  description?: string;
  category_id?: string;
  tags?: string[];
}

// 更新任务请求类型
export interface UpdateTaskRequest {
  name?: string;
  description?: string;
  category_id?: string;
  tags?: string[];
}

// 应用配置类型
export interface AppConfig {
  theme: 'light' | 'dark' | 'system';
  language: string;
  notifications: {
    enabled: boolean;
    sound: boolean;
  };
  timer: {
    auto_start: boolean;
    reminder_interval: number;
  };
} 