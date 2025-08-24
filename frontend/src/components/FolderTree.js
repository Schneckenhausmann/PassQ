import React, { useState, useEffect } from 'react';
import { Icons } from './Icons';
import FolderIcon from './FolderIcon';

function FolderTree({ folders, onAddFolder, onDeleteFolder, onRenameFolder, onMoveEntry, onMoveFolder, selectedFolder, onSelectFolder }) {
  const [newFolderName, setNewFolderName] = useState('');
  const [showAddForm, setShowAddForm] = useState(false);
  const [dragOverFolder, setDragOverFolder] = useState(null);
  
  // Load expanded folders from localStorage or default to root expanded
  const [expandedFolders, setExpandedFolders] = useState(() => {
    try {
      const saved = localStorage.getItem('passq-expanded-folders');
      return saved ? new Set(JSON.parse(saved)) : new Set(['root']);
    } catch {
      return new Set(['root']);
    }
  });
  
  const [addingSubfolderTo, setAddingSubfolderTo] = useState(null);
  const [renamingFolder, setRenamingFolder] = useState(null);
  const [renameValue, setRenameValue] = useState('');

  // Save expanded folders to localStorage whenever it changes
  useEffect(() => {
    try {
      localStorage.setItem('passq-expanded-folders', JSON.stringify([...expandedFolders]));
    } catch (error) {
      console.error('Failed to save expanded folders to localStorage:', error);
    }
  }, [expandedFolders]);

  // Auto-expand folders that have subfolders when folders change
  useEffect(() => {
    const foldersWithSubfolders = folders.filter(folder => 
      folders.some(subfolder => subfolder.parentId === folder.id)
    );
    
    if (foldersWithSubfolders.length > 0) {
      setExpandedFolders(prev => {
        const newExpanded = new Set(prev);
        foldersWithSubfolders.forEach(folder => {
          if (folder.id !== 'root') {
            newExpanded.add(folder.id);
          }
        });
        return newExpanded;
      });
    }
  }, [folders]);

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
          className={`folder-item flex items-center justify-between py-1.5 px-2 rounded hover:bg-gray-50 cursor-pointer transition-all ${
            selectedFolder === folder.id ? 'bg-gray-100 border-l-4 border-gray-500' : 'border-l-4 border-transparent'
          } ${
            dragOverFolder === folder.id ? 'bg-gray-50 border-l-4 border-gray-300' : ''
          }`}
          style={{ paddingLeft: `${8 + level * 20}px` }}
          draggable={folder.id !== 'root'}
          onDragStart={(e) => handleFolderDragStart(e, folder)}
          onDragOver={(e) => handleDragOver(e, folder.id)}
          onDragLeave={handleDragLeave}
          onDrop={(e) => handleDrop(e, folder.id)}
          onClick={() => onSelectFolder(folder.id)}
        >
          <div className="flex items-center gap-2 flex-1">
            {/* Expand/collapse button */}
            {hasSubfolders && (
              <button 
                className={`w-4 h-4 flex items-center justify-center rounded text-xs hover:bg-gray-200 transition-colors ${
                  selectedFolder === folder.id ? 'text-gray-700' : 'text-gray-600'
                }`}
                onClick={(e) => {
                  e.stopPropagation();
                  toggleFolder(folder.id);
                }}
              >
                {isExpanded ? 
                  <span>▼</span> : 
                  <span>▶</span>
                }
              </button>
            )}
            {!hasSubfolders && <div className="w-4" />}
            
            {/* Folder icon */}
            <div className={`flex-shrink-0 mr-2 ${
              selectedFolder === folder.id ? 'text-gray-600' : 'text-gray-500'
            }`}>
              <FolderIcon isOpen={isExpanded && hasSubfolders} />
            </div>
            
            {/* Folder name and count */}
            {renamingFolder === folder.id ? (
              <form onSubmit={handleRenameSubmit} className="flex items-center gap-2 flex-1">
                <input
                  type="text"
                  value={renameValue}
                  onChange={(e) => setRenameValue(e.target.value)}
                  className="flex-1 px-2 py-1 text-sm border border-gray-300 rounded bg-white text-gray-900 focus:border-gray-500 focus:outline-none"
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
                <span className={`font-medium text-sm break-words ${
                  selectedFolder === folder.id ? 'text-gray-800' : 'text-gray-800'
                }`}>{folder.name}</span>
                <span className={`text-xs flex-shrink-0 ml-2 ${
                  selectedFolder === folder.id ? 'text-gray-600' : 'text-gray-500'
                }`}>({folder.entryCount || 0})</span>
              </>
            )}
          </div>
          
          {/* Action buttons */}
          {renamingFolder !== folder.id && (
            <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
              <button 
                className="w-5 h-5 flex items-center justify-center text-xs rounded hover:bg-gray-200 text-gray-600 hover:text-gray-800 transition-colors"
                onClick={(e) => {
                  e.stopPropagation();
                  setAddingSubfolderTo(folder.id);
                  setShowAddForm(true);
                  setNewFolderName('');
                }}
                title="Add subfolder"
              >
                +
              </button>
              {folder.id !== 'root' && (
                <>
                  <button 
                    className="w-5 h-5 flex items-center justify-center text-xs rounded hover:bg-gray-200 text-gray-600 hover:text-gray-800 transition-colors"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRenameStart(folder);
                    }}
                    title="Rename folder"
                  >
                    ✎
                  </button>
                  <button 
                    className="w-5 h-5 flex items-center justify-center text-xs rounded hover:bg-gray-200 text-red-600 hover:text-red-800 transition-colors"
                    onClick={(e) => {
                      e.stopPropagation();
                      onDeleteFolder(folder.id);
                    }}
                    title="Delete folder"
                  >
                    ×
                  </button>
                </>
              )}
            </div>
          )}
        </div>
        
        {addingSubfolderTo === folder.id && showAddForm && (
          <form 
            onSubmit={(e) => handleAddFolder(e, folder.id)} 
            className="py-2 bg-gray-50 border-l-4 border-gray-200 rounded-r"
            style={{ paddingLeft: `${8 + (level + 1) * 20}px` }}
          >
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder="Subfolder name"
                value={newFolderName}
                onChange={(e) => setNewFolderName(e.target.value)}
                className="flex-1 px-2 py-1 text-sm border border-gray-300 rounded bg-white text-gray-900 focus:border-gray-500 focus:outline-none"
                autoFocus
              />
              <button 
                type="submit" 
                className="px-2 py-1 text-xs bg-gray-600 text-white rounded hover:bg-gray-700 transition-colors"
              >
                Add
              </button>
              <button 
                type="button" 
                className="px-2 py-1 text-xs bg-gray-300 text-gray-700 rounded hover:bg-gray-400 transition-colors"
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

  const handleFolderDragStart = (e, folder) => {
    e.stopPropagation();
    e.dataTransfer.setData('text/plain', JSON.stringify({ type: 'folder', ...folder }));
    e.dataTransfer.effectAllowed = 'move';
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
      const dragData = JSON.parse(e.dataTransfer.getData('text/plain'));
      
      if (dragData.type === 'folder') {
        // Handle folder drag and drop
        if (dragData.id && dragData.id !== targetFolderId && dragData.parentId !== targetFolderId) {
          // Prevent dropping a folder into itself or its descendants
          if (!isDescendantFolder(targetFolderId, dragData.id)) {
            onMoveFolder && onMoveFolder(dragData.id, targetFolderId);
          }
        }
      } else {
        // Handle password entry drag and drop
        if (dragData.id && dragData.folderId !== targetFolderId) {
          onMoveEntry(dragData.id, targetFolderId);
        }
      }
    } catch (error) {
      console.error('Error parsing dropped data:', error);
    }
  };

  const isDescendantFolder = (potentialParentId, folderId) => {
    const folder = folders.find(f => f.id === potentialParentId);
    if (!folder) return false;
    if (folder.parentId === folderId) return true;
    if (folder.parentId) return isDescendantFolder(folder.parentId, folderId);
    return false;
  };

  const topLevelFolders = folders.filter(folder => !folder.parentId || folder.parentId === 'root');
  const isRootExpanded = expandedFolders.has('root');
  const hasTopLevelFolders = topLevelFolders.length > 0;
  
  // Calculate total items count for root folder
  const totalItemsCount = folders.reduce((sum, folder) => sum + (folder.entryCount || 0), 0);

  return (
    <div className="folder-tree">
      <ul className="folder-list">
        {/* Root folder */}
        <li key="root" className="folder-item-container group">
          <div 
            className={`folder-item flex items-center justify-between py-1.5 px-2 rounded hover:bg-gray-50 cursor-pointer transition-all ${
              selectedFolder === 'root' ? 'bg-gray-100 border-l-4 border-gray-500' : 'border-l-4 border-transparent'
            }`}
            onClick={() => onSelectFolder('root')}
          >
            <div className="flex items-center gap-2 flex-1">
              {/* Expand/collapse button for root */}
              {hasTopLevelFolders && (
                <button 
                  className={`w-4 h-4 flex items-center justify-center rounded text-xs hover:bg-gray-200 transition-colors ${
                    selectedFolder === 'root' ? 'text-gray-700' : 'text-gray-600'
                  }`}
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleFolder('root');
                  }}
                >
                  {isRootExpanded ? 
                    <span>▼</span> : 
                    <span>▶</span>
                  }
                </button>
              )}
              {!hasTopLevelFolders && <div className="w-4" />}
              
              {/* Root folder icon */}
              <div className={`flex-shrink-0 mr-2 ${
                selectedFolder === 'root' ? 'text-gray-600' : 'text-gray-500'
              }`}>
                <FolderIcon isOpen={isRootExpanded && hasTopLevelFolders} />
              </div>
              
              {/* Root folder name and count */}
              <span className={`font-medium text-sm break-words ${
                selectedFolder === 'root' ? 'text-gray-800' : 'text-gray-800'
              }`}>All Items</span>
              <span className={`text-xs flex-shrink-0 ml-2 ${
                selectedFolder === 'root' ? 'text-gray-600' : 'text-gray-500'
              }`}>({totalItemsCount})</span>
            </div>
          </div>
          
          {/* Render child folders when root is expanded */}
          {isRootExpanded && hasTopLevelFolders && (
            <ul className="ml-6">
              {topLevelFolders.map(folder => renderFolder(folder, 0))}
            </ul>
          )}
        </li>
      </ul>
    </div>
  );
}

export default FolderTree;