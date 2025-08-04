import React, { useState } from 'react';
import { Icons } from './Icons';

function FolderTree({ folders, onAddFolder, onDeleteFolder, onMoveEntry, selectedFolder, onSelectFolder }) {
  const [newFolderName, setNewFolderName] = useState('');
  const [showAddForm, setShowAddForm] = useState(false);
  const [dragOverFolder, setDragOverFolder] = useState(null);
  const [expandedFolders, setExpandedFolders] = useState(new Set(['root']));
  const [addingSubfolderTo, setAddingSubfolderTo] = useState(null);

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

  const getSubfolders = (parentId) => {
    return folders.filter(folder => folder.parentId === parentId);
  };

  const renderFolder = (folder, level = 0) => {
    const subfolders = getSubfolders(folder.id);
    const isExpanded = expandedFolders.has(folder.id);
    const hasSubfolders = subfolders.length > 0;

    return (
      <li key={folder.id} className="folder-item-container">
        <div 
          className={`folder-item ${
            selectedFolder === folder.id ? 'selected' : ''
          } ${
            dragOverFolder === folder.id ? 'drag-over' : ''
          }`}
          style={{ paddingLeft: `${level * 20 + 12}px` }}
          onDragOver={(e) => handleDragOver(e, folder.id)}
          onDragLeave={handleDragLeave}
          onDrop={(e) => handleDrop(e, folder.id)}
        >
          <div className="folder-content" onClick={() => onSelectFolder(folder.id)}>
            {hasSubfolders && (
              <button 
                className="folder-toggle"
                onClick={(e) => {
                  e.stopPropagation();
                  toggleFolder(folder.id);
                }}
              >
                {isExpanded ? 
                  <Icons.ChevronDown size={12} /> : 
                  <Icons.ChevronRight size={12} />
                }
              </button>
            )}
            {!hasSubfolders && <div className="folder-spacer" />}
            <span className="folder-icon">
              {isExpanded && hasSubfolders ? 
                <Icons.FolderOpen size={16} /> : 
                <Icons.Folder size={16} />
              }
            </span>
            <span className="folder-name">{folder.name}</span>
            <span className="entry-count">({folder.entryCount || 0})</span>
          </div>
          <div className="folder-actions">
            <button 
              className="add-subfolder-btn"
              onClick={(e) => {
                e.stopPropagation();
                setAddingSubfolderTo(folder.id);
                setShowAddForm(true);
                setNewFolderName('');
              }}
              title="Add subfolder"
            >
              <Icons.FolderPlus size={12} />
            </button>
            {folder.id !== 'root' && (
              <button 
                className="delete-folder-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  if (window.confirm(`Delete folder "${folder.name}"? All entries will be moved to the parent folder.`)) {
                    onDeleteFolder(folder.id);
                  }
                }}
                title="Delete folder"
              >
                <Icons.Trash size={12} />
              </button>
            )}
          </div>
        </div>
        
        {addingSubfolderTo === folder.id && showAddForm && (
          <form 
            onSubmit={(e) => handleAddFolder(e, folder.id)} 
            className="add-folder-form"
            style={{ paddingLeft: `${(level + 1) * 20 + 12}px` }}
          >
            <input
              type="text"
              placeholder="Subfolder name"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              autoFocus
            />
            <div className="form-actions">
              <button type="submit" className="btn-primary">Add</button>
              <button 
                type="button" 
                className="btn-secondary"
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
      <div className="folder-header">
        <h3>Folders</h3>
        <button 
          className="btn-secondary add-folder-btn"
          onClick={() => {
            setShowAddForm(!showAddForm);
            setAddingSubfolderTo(null);
            setNewFolderName('');
          }}
          title="Add new folder"
        >
          <Icons.FolderPlus size={16} />
        </button>
      </div>
      
      {showAddForm && !addingSubfolderTo && (
        <form onSubmit={(e) => handleAddFolder(e)} className="add-folder-form">
          <input
            type="text"
            placeholder="Folder name"
            value={newFolderName}
            onChange={(e) => setNewFolderName(e.target.value)}
            autoFocus
          />
          <div className="form-actions">
            <button type="submit" className="btn-primary">Add</button>
            <button 
              type="button" 
              className="btn-secondary"
              onClick={() => {
                setShowAddForm(false);
                setNewFolderName('');
              }}
            >
              Cancel
            </button>
          </div>
        </form>
      )}
      
      <ul className="folder-list">
        {folders.filter(folder => !folder.parentId || folder.parentId === 'root').map(folder => renderFolder(folder))}
      </ul>
    </div>
  );
}

export default FolderTree;