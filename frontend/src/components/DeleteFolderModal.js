import React, { useState } from 'react';
import { Icons } from './Icons';

function DeleteFolderModal({ isOpen, onClose, onConfirm, folder, folders, passwords }) {
  const [migrateTo, setMigrateTo] = useState('');
  const [customFolderId, setCustomFolderId] = useState('');

  if (!isOpen || !folder) return null;

  // Get passwords in this folder
  const folderPasswords = passwords.filter(p => p.folder_id === folder.id);
  const hasPasswords = folderPasswords.length > 0;

  // Get available folders for migration (excluding the folder being deleted and its subfolders)
  const availableFolders = folders.filter(f => 
    f.id !== folder.id && 
    f.id !== 'root' && 
    !f.parent_folder_id || f.parent_folder_id !== folder.id
  );

  const handleConfirm = () => {
    let targetFolderId = null;
    
    if (hasPasswords && migrateTo === 'custom' && customFolderId) {
      targetFolderId = customFolderId;
    } else if (hasPasswords && migrateTo === 'parent') {
      targetFolderId = folder.parent_folder_id || null; // null means root
    } else if (hasPasswords && migrateTo === 'delete') {
      targetFolderId = 'DELETE_PASSWORDS'; // Special flag to indicate passwords should be deleted
    }
    
    onConfirm(folder.id, targetFolderId);
  };

  const handleMigrateToChange = (value) => {
    setMigrateTo(value);
    if (value === 'custom' && availableFolders.length > 0) {
      setCustomFolderId(availableFolders[0].id);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-sm" onClick={onClose}>
      <div className="relative bg-white rounded-xl shadow-2xl cartoon-shadow p-6 w-full max-w-md mx-auto cartoon-border" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-xl font-bold text-red-600">Delete Folder</h3>
          <button className="text-black/60 hover:text-black transition" onClick={onClose}>
            <Icons.Close size={20} />
          </button>
        </div>
        
        <div className="space-y-4">
          <div className="flex items-center gap-3 p-3 bg-red-50 rounded-lg border border-red-200">
            <Icons.AlertTriangle size={24} className="text-red-500 flex-shrink-0" />
            <div>
              <p className="font-medium text-red-800">Are you sure you want to delete this folder?</p>
              <p className="text-sm text-red-600 mt-1">
                Folder: <span className="font-medium">{folder.name}</span>
              </p>
            </div>
          </div>

          {hasPasswords && (
            <div className="space-y-3">
              <div className="p-3 bg-yellow-50 rounded-lg border border-yellow-200">
                <div className="flex items-center gap-2 mb-2">
                  <Icons.Key size={16} className="text-yellow-600" />
                  <span className="font-medium text-yellow-800">
                    This folder contains {folderPasswords.length} password{folderPasswords.length !== 1 ? 's' : ''}
                  </span>
                </div>
                <p className="text-sm text-yellow-700">
                  Choose what to do with the passwords:
                </p>
              </div>

              <div className="space-y-2">
                <label className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="migrateTo"
                    value="delete"
                    checked={migrateTo === 'delete'}
                    onChange={(e) => handleMigrateToChange(e.target.value)}
                    className="w-4 h-4 text-red-600"
                  />
                  <span className="text-sm text-red-700">
                    Delete all passwords permanently
                  </span>
                </label>

                <label className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="migrateTo"
                    value="parent"
                    checked={migrateTo === 'parent'}
                    onChange={(e) => handleMigrateToChange(e.target.value)}
                    className="w-4 h-4 text-blue-600"
                  />
                  <span className="text-sm">
                    Move to parent folder ({folder.parent_folder_id ? 
                      folders.find(f => f.id === folder.parent_folder_id)?.name || 'Unknown' : 
                      'All Items'
                    })
                  </span>
                </label>

                {availableFolders.length > 0 && (
                  <label className="flex items-center gap-2">
                    <input
                      type="radio"
                      name="migrateTo"
                      value="custom"
                      checked={migrateTo === 'custom'}
                      onChange={(e) => handleMigrateToChange(e.target.value)}
                      className="w-4 h-4 text-blue-600"
                    />
                    <span className="text-sm">Move to a different folder:</span>
                  </label>
                )}

                {migrateTo === 'custom' && availableFolders.length > 0 && (
                  <div className="ml-6">
                    <select
                      value={customFolderId}
                      onChange={(e) => setCustomFolderId(e.target.value)}
                      className="w-full border-2 border-black rounded px-3 py-2 text-sm"
                    >
                      {availableFolders.map(folder => (
                        <option key={folder.id} value={folder.id}>
                          {folder.name}
                        </option>
                      ))}
                    </select>
                  </div>
                )}

                {availableFolders.length === 0 && migrateTo === 'custom' && (
                  <p className="text-sm text-gray-500 ml-6">
                    No other folders available for migration.
                  </p>
                )}
              </div>
            </div>
          )}

          <div className="flex justify-end gap-2 mt-6">
            <button 
              type="button" 
              className="cartoon-btn" 
              onClick={onClose}
            >
              Cancel
            </button>
            <button 
              type="button" 
              className={`cartoon-btn text-white border-red-500 ${
                hasPasswords && !migrateTo 
                  ? 'bg-gray-400 border-gray-400 cursor-not-allowed' 
                  : 'bg-red-500 hover:bg-red-600 hover:border-red-600'
              }`}
              onClick={handleConfirm}
              disabled={hasPasswords && !migrateTo}
            >
              Delete Folder
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default DeleteFolderModal;