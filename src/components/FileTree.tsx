import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useFileStore } from "../stores/fileStore";

interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

type ExpandedMap = Record<string, boolean>;

export function FileTree() {
  const openFileByPath = useFileStore((s) => s.openFileByPath);
  const [rootEntries, setRootEntries] = useState<FileEntry[]>([]);
  const [expanded, setExpanded] = useState<ExpandedMap>({});
  const [children, setChildren] = useState<Record<string, FileEntry[]>>({});

  // 加载根目录
  useEffect(() => {
    invoke<FileEntry[]>("list_directory", { path: "" })
      .then(setRootEntries)
      .catch(console.error);
  }, []);

  const toggleDir = useCallback(
    async (dirPath: string) => {
      if (expanded[dirPath]) {
        // 折叠
        setExpanded((prev) => ({ ...prev, [dirPath]: false }));
      } else {
        // 展开（加载子目录）
        if (!children[dirPath]) {
          try {
            const entries = await invoke<FileEntry[]>("list_directory", {
              path: dirPath,
            });
            setChildren((prev) => ({ ...prev, [dirPath]: entries }));
          } catch (e) {
            console.error(e);
          }
        }
        setExpanded((prev) => ({ ...prev, [dirPath]: true }));
      }
    },
    [expanded, children],
  );

  const renderEntry = (entry: FileEntry, depth: number) => {
    const isExpanded = expanded[entry.path];
    const childEntries = children[entry.path];

    return (
      <div key={entry.path}>
        <div
          className={`file-item ${entry.is_dir ? "file-dir" : "file-file"}`}
          style={{ paddingLeft: 8 + depth * 14 }}
          onClick={() =>
            entry.is_dir
              ? toggleDir(entry.path)
              : openFileByPath(entry.path, entry.name)
          }
        >
          <span className="file-icon">
            {entry.is_dir ? (isExpanded ? "📂" : "📁") : "📄"}
          </span>
          <span className="file-name">{entry.name}</span>
        </div>

        {/* 子目录 */}
        {entry.is_dir && isExpanded && childEntries && (
          <div className="file-children">
            {childEntries.map((child) => renderEntry(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="file-tree">
      {rootEntries.map((entry) => renderEntry(entry, 0))}
    </div>
  );
}
