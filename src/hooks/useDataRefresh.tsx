import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef } from "react";
import { useState } from "react";

type DataChangeType = 
  | "all_data_cleared" 
  | "sync_completed" 
  | "conflicts_resolved" 
  | "data_imported" 
  | "database_restored"
  | "task_created"
  | "task_updated"
  | "task_deleted"
  | "category_created"
  | "category_updated"
  | "category_deleted"
  | "transaction_created"
  | "transaction_updated"
  | "transaction_deleted"
  | "timer_started"
  | "timer_stopped"
  | "timer_updated";

interface DataRefreshOptions {
  // 需要刷新的数据类型
  refreshTypes?: DataChangeType[];
  // 自定义刷新函数
  onRefresh?: (changeType: DataChangeType) => void;
  // 是否启用自动刷新（默认为true）
  enabled?: boolean;
}

/**
 * 数据刷新Hook
 * 监听Tauri事件，当数据发生变化时自动触发刷新
 */
export function useDataRefresh(
  refreshCallback: () => void | Promise<void>,
  options: DataRefreshOptions = {}
) {
  const { refreshTypes, onRefresh, enabled = true } = options;
  const refreshRef = useRef(refreshCallback);
  
  // 更新回调引用
  useEffect(() => {
    refreshRef.current = refreshCallback;
  }, [refreshCallback]);

  const handleDataChange = useCallback(async (changeType: DataChangeType) => {
    console.log(`数据变化检测: ${changeType}`);
    
    // 如果指定了特定的刷新类型，检查是否匹配
    if (refreshTypes && !refreshTypes.includes(changeType)) {
      return;
    }
    
    try {
      // 执行自定义回调
      if (onRefresh) {
        onRefresh(changeType);
      }
      
      // 执行刷新回调
      await refreshRef.current();
      console.log(`数据刷新完成: ${changeType}`);
    } catch (error) {
      console.error(`数据刷新失败: ${changeType}`, error);
    }
  }, [refreshTypes, onRefresh]);

  useEffect(() => {
    if (!enabled) return;

    let unlistenDataChanged: (() => void) | null = null;
    let unlistenSyncStatus: (() => void) | null = null;

    // 监听数据变化事件
    const setupDataChangeListener = async () => {
      try {
        unlistenDataChanged = await listen<DataChangeType>("data_changed", (event) => {
          handleDataChange(event.payload);
        });
        console.log("数据变化监听器已设置");
      } catch (error) {
        console.error("设置数据变化监听器失败:", error);
      }
    };

    // 监听同步状态变化事件（可选）
    const setupSyncStatusListener = async () => {
      try {
        unlistenSyncStatus = await listen<string>("sync_status_changed", (event) => {
          console.log(`同步状态变化: ${event.payload}`);
          // 可以根据需要处理同步状态变化
        });
        console.log("同步状态监听器已设置");
      } catch (error) {
        console.error("设置同步状态监听器失败:", error);
      }
    };

    setupDataChangeListener();
    setupSyncStatusListener();

    // 清理函数
    return () => {
      if (unlistenDataChanged) {
        unlistenDataChanged();
        console.log("数据变化监听器已清理");
      }
      if (unlistenSyncStatus) {
        unlistenSyncStatus();
        console.log("同步状态监听器已清理");
      }
    };
  }, [enabled, handleDataChange]);
}

/**
 * 通用数据刷新Hook
 * 适用于需要在数据变化时刷新的组件
 */
export function useAutoRefresh<T>(
  fetchFunction: () => Promise<T>,
  dependencies: any[] = [],
  options: DataRefreshOptions = {}
) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await fetchFunction();
      setData(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error("数据获取失败:", err);
    } finally {
      setLoading(false);
    }
  }, dependencies);

  // 初始数据获取
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // 设置数据刷新监听
  useDataRefresh(fetchData, options);

  return { data, loading, error, refetch: fetchData };
} 