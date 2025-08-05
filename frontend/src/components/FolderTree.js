import React, { useState } from 'react';
import { Icons } from './Icons';

function FolderTree({ folders, onAddFolder, onDeleteFolder, onRenameFolder, onMoveEntry, selectedFolder, onSelectFolder }) {
  const [newFolderName, setNewFolderName] = useState('');
  const [showAddForm, setShowAddForm] = useState(false);
  const [dragOverFolder, setDragOverFolder] = useState(null);
  const [expandedFolders, setExpandedFolders] = useState(new Set(['root']));
  const [addingSubfolderTo, setAddingSubfolderTo] = useState(null);
  const [renamingFolder, setRenamingFolder] = useState(null);
  const [renameValue, setRenameValue] = useState('');

  const handleAddFolder = (e, parentId = null) => {
    e.preventDefault();
    if (newFolderName.trim()) {
      onAddFolder(newFolderName.trim(), parentId);
      setNewFolderName('');
      setShowAddForm(false);
      setAddingSubfolderTo(null);
    }
  };

  const toggleFolder = (folderId) => {
    const newExpanded = new Set(expandedFolders);
    if (newExpanded.has(folderId)) {
      newExpanded.delete(folderId);
    } else {
      newExpanded.add(folderId);
    }
    setExpandedFolders(newExpanded);
  };

  const handleRenameStart = (folder) => {
    setRenamingFolder(folder.id);
    setRenameValue(folder.name);
  };

  const handleRenameSubmit = (e) => {
    e.preventDefault();
    if (renameValue.trim() && renamingFolder) {
      onRenameFolder(renamingFolder, renameValue.trim());
      setRenamingFolder(null);
      setRenameValue('');
    }
  };

  const handleRenameCancel = () => {
    setRenamingFolder(null);
    setRenameValue('');
  };

  const getSubfolders = (parentId) => {
    return folders.filter(folder => folder.parentId === parentId);
  };

  const renderFolder = (folder, level = 0) => {
    const subfolders = getSubfolders(folder.id);
    const isExpanded = expandedFolders.has(folder.id);
    const hasSubfolders = subfolders.length > 0;

    return (
      <li key={folder.id} className="folder-item-container group">
        <div 
          className={`folder-item flex items-center justify-between py-2 px-3 border border-transparent hover:border-black hover:bg-gray-100 cursor-pointer transition-all ${
            selectedFolder === folder.id ? 'bg-black text-white border-black' : 'text-black'
          } ${
            dragOverFolder === folder.id ? 'bg-gray-200 border-black' : ''
          }`}
          style={{ marginLeft: `${level * 16}px` }}
          onDragOver={(e) => handleDragOver(e, folder.id)}
          onDragLeave={handleDragLeave}
          onDrop={(e) => handleDrop(e, folder.id)}
          onClick={() => onSelectFolder(folder.id)}
        >
          <div className="flex items-center gap-2 flex-1 min-w-0">
            {/* Tree structure indicator - markdown style */}
            {level > 0 && (
              <div className="flex items-center text-black font-mono text-xs">
                {Array.from({ length: level }, (_, i) => (
                  <span key={i} className="w-4">
                    {i === level - 1 ? '├─' : '│ '}
                  </span>
                ))}
              </div>
            )}
            
            {/* Expand/collapse button */}
            {hasSubfolders && (
              <button 
                className={`w-4 h-4 flex items-center justify-center hover:bg-gray-200 rounded ${
                  selectedFolder === folder.id ? 'text-white hover:bg-gray-700' : 'text-black'
                }`}
                onClick={(e) => {
                  e.stopPropagation();
                  toggleFolder(folder.id);
                }}
              >
                {isExpanded ? 
                  <span className="text-xs font-bold">−</span> : 
                  <span className="text-xs font-bold">+</span>
                }
              </button>
            )}
            {!hasSubfolders && <div className="w-4" />}
            
            {/* Folder icon */}
            <span className={`flex-shrink-0 font-bold text-sm ${
              selectedFolder === folder.id ? 'text-white' : 'text-black'
            }`}>
              {isExpanded && hasSubfolders ? '▼' : '▶'}
            </span>
            
            {/* Folder name and count */}
            {renamingFolder === folder.id ? (
              <form onSubmit={handleRenameSubmit} className="flex items-center gap-2 flex-1">
                <input
                  type="text"
                  value={renameValue}
                  onChange={(e) => setRenameValue(e.target.value)}
                  className="flex-1 px-2 py-1 text-sm border border-black bg-white text-black"
                  autoFocus
                  onBlur={handleRenameCancel}
                  onKeyDown={(e) => {
                    if (e.key === 'Escape') {
                      handleRenameCancel();
                    }
                  }}
                />
              </form>
            ) : (
              <>
                <span className={`font-medium truncate ${
                  selectedFolder === folder.id ? 'text-white' : 'text-black'
                }`}>{folder.name}</span>
                <span className={`text-xs flex-shrink-0 ${
                  selectedFolder === folder.id ? 'text-gray-300' : 'text-gray-600'
                }`}>({folder.entryCount || 0})</span>
              </>
            )}
          </div>
          
          {/* Action buttons */}
          {renamingFolder !== folder.id && (
            <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button 
                className={`p-1 border border-transparent hover:border-black hover:bg-gray-200 transition-all ${
                  selectedFolder === folder.id ? 'text-white hover:bg-gray-700 hover:border-white' : 'text-black'
                }`}
                onClick={(e) => {
                  e.stopPropagation();
                  setAddingSubfolderTo(folder.id);
                  setShowAddForm(true);
                  setNewFolderName('');
                }}
                title="Add subfolder"
              >
                <span className="text-xs font-bold">+</span>
              </button>
              {folder.id !== 'root' && (
                <>
                  <button 
                    className={`p-1 border border-transparent hover:border-black hover:bg-gray-200 transition-all ${
                      selectedFolder === folder.id ? 'text-white hover:bg-gray-700 hover:border-white' : 'text-black'
                    }`}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRenameStart(folder);
                    }}
                    title="Rename folder"
                  >
                    <span className="text-xs font-bold">✎</span>
                  </button>
                  <button 
                    className={`p-1 border border-transparent hover:border-black hover:bg-gray-200 transition-all ${
                      selectedFolder === folder.id ? 'text-white hover:bg-gray-700 hover:border-white' : 'text-black'
                    }`}
                    onClick={(e) => {
                      e.stopPropagation();
                      if (window.confirm(`Delete folder "${folder.name}"? All entries will be moved to the parent folder.`)) {
                        onDeleteFolder(folder.id);
                      }
                    }}
                    title="Delete folder"
                  >
                    <span className="text-xs font-bold">×</span>
                  </button>
                </>
              )}
            </div>
          )}
        </div>
        
        {addingSubfolderTo === folder.id && showAddForm && (
          <form 
            onSubmit={(e) => handleAddFolder(e, folder.id)} 
            className="py-2 px-3 border border-black bg-gray-50"
            style={{ marginLeft: `${(level + 1) * 16}px` }}
          >
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder="Subfolder name"
                value={newFolderName}
                onChange={(e) => setNewFolderName(e.target.value)}
                className="flex-1 px-2 py-1 text-sm border border-black bg-white text-black"
                autoFocus
              />
              <button 
                type="submit" 
                className="px-3 py-1 text-xs font-bold border border-black bg-black text-white hover:bg-gray-800 transition-colors"
              >
                Add
              </button>
              <button 
                type="button" 
                className="px-3 py-1 text-xs font-bold border border-black bg-white text-black hover:bg-gray-100 transition-colors"
                onClick={() => {
                  setShowAddForm(false);
                  setNewFolderName('');
                  setAddingSubfolderTo(null);
                }}
              >
                Cancel
              </button>
            </div>
          </form>
        )}
        
        {isExpanded && hasSubfolders && (
          <ul className="subfolder-list">
            {subfolders.map(subfolder => renderFolder(subfolder, level + 1))}
          </ul>
        )}
      </li>
    );
  };

  const handleDragOver = (e, folderId) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverFolder(folderId);
  };

  const handleDragLeave = () => {
    setDragOverFolder(null);
  };

  const handleDrop = (e, targetFolderId) => {
    e.preventDefault();
    setDragOverFolder(null);
    
    try {
      const entryData = JSON.parse(e.dataTransfer.getData('text/plain'));
      if (entryData.id && entryData.folderId !== targetFolderId) {
        onMoveEntry(entryData.id, targetFolderId);
      }
    } catch (error) {
      console.error('Error parsing dropped data:', error);
    }
  };

  return (
    <div className="folder-tree">
      <ul className="folder-list">
        {folders.filter(folder => !folder.parentId || folder.parentId === 'root').map(folder => renderFolder(folder))}
      </ul>
    </div>
  );
}

export default FolderTree;