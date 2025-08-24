import React, { useState, useEffect } from 'react';
import PasswordItem from './PasswordItem';
import FolderTree from './FolderTree';
import AddEntryModal from './AddEntryModal';
import EditEntryModal from './EditEntryModal';
import ShareModal from './ShareModal';
import SearchBar from './SearchBar';
import { Icons } from './Icons';
import Logo from './Logo';
import { passwordAPI, folderAPI, csvAPI } from '../services/api';

function Dashboard() {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [passwords, setPasswords] = useState([]);
  const [folders, setFolders] = useState([
    { id: 'root', name: 'All Items', entryCount: 0, parentId: null },
  ]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [selectedFolder, setSelectedFolder] = useState('root');
  const [isShareModalOpen, setShareModalOpen] = useState(false);
  const [shareItem, setShareItem] = useState(null);
  const [addModalOpen, setAddModalOpen] = useState(false);
  const [editModalOpen, setEditModalOpen] = useState(false);
  const [editingEntry, setEditingEntry] = useState(null);
  const [sharedPasswords, setSharedPasswords] = useState([]);
  const [showSharedItems, setShowSharedItems] = useState(false);
  const [searchResults, setSearchResults] = useState([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [isSearchActive, setIsSearchActive] = useState(false);

  // Load data from API
  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [passwordsResponse, foldersResponse] = await Promise.all([
        passwordAPI.getAll(),
        folderAPI.getAll()
      ]);
      
      console.log('Passwords API Response:', passwordsResponse.data);
      console.log('Raw passwords data:', passwordsResponse.data.data);
      
      setPasswords(passwordsResponse.data.data || []);
      const apiFolders = foldersResponse.data.data || [];
      const allFolders = [
        { id: 'root', name: 'All Items', entryCount: 0, parentId: null },
        ...apiFolders
      ];
      setFolders(allFolders);
      updateFolderCounts(passwordsResponse.data.data || [], allFolders);
      
      // Load shared items
      await loadSharedItems();
    } catch (err) {
      console.error('Error loading data:', err);
      setError('Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const loadSharedItems = async () => {
    try {
      const response = await passwordAPI.getSharedPasswords();
      setSharedPasswords(response.data.data || []);
    } catch (err) {
      console.error('Error loading shared items:', err);
    }
  };

  const handleShareItem = (itemId, itemType) => {
    setShareItem({ id: itemId, type: itemType });
    setShareModalOpen(true);
  };

  const handleShareSuccess = () => {
    setShareModalOpen(false);
    setShareItem(null);
    // Optionally reload shared items
    loadSharedItems();
  };

  // CSV Export/Import handlers
  const handleExportCSV = async () => {
    try {
      const response = await csvAPI.export();
      
      // Create blob and download
      const blob = new Blob([response.data], { type: 'text/csv' });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = 'passq_export.csv';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Error exporting CSV:', err);
      setError('Failed to export passwords');
    }
  };

  const handleImportCSV = async (event) => {
    const file = event.target.files[0];
    if (!file) return;
    
    try {
      const text = await file.text();
      const response = await csvAPI.import(text);
      
      // Reload data to show imported passwords
      await loadData();
      
      // Show success message
      const importedCount = response.data.data;
      alert(`Successfully imported ${importedCount} passwords`);
    } catch (err) {
      console.error('Error importing CSV:', err);
      setError('Failed to import passwords. Please check the CSV format.');
    }
    
    // Reset file input
    event.target.value = '';
  };

  const triggerFileInput = () => {
    document.getElementById('csv-import-input').click();
  };

  // Update folder entry counts
  const updateFolderCounts = (updatedPasswords, updatedFolders) => {
    const counts = {};
    updatedPasswords.forEach(password => {
      // Use folder_id from API response
      const folderId = password.folder_id || 'root';
      counts[folderId] = (counts[folderId] || 0) + 1;
    });
    
    const foldersWithCounts = updatedFolders.map(folder => ({
      ...folder,
      entryCount: counts[folder.id] || 0
    }));
    
    setFolders(foldersWithCounts);
  };

  // Folder management
  const handleAddFolder = async (name, parentId = null) => {
    try {
      const folderData = {
        name,
        parent_folder_id: parentId === 'root' ? null : parentId,
        user_id: 'current-user-id' // This should come from auth context
      };
      
      const response = await folderAPI.create(folderData);
      const newFolder = response.data.data;
      
      const updatedFolders = [...folders, newFolder];
      setFolders(updatedFolders);
      updateFolderCounts(passwords, updatedFolders);
    } catch (err) {
      console.error('Error creating folder:', err);
      setError('Failed to create folder');
    }
  };

  const handleDeleteFolder = async (folderId) => {
    if (folderId === 'root') return;
    
    try {
      await folderAPI.delete(folderId);
      
      const folderToDelete = folders.find(f => f.id === folderId);
      const parentId = folderToDelete?.parentId || 'root';
      
      // Move all entries from deleted folder to parent folder
      const updatedPasswords = passwords.map(password => 
        password.folder_id === folderId ? { ...password, folder_id: parentId } : password
      );
      
      // Move all subfolders to parent folder
      const updatedFolders = folders
        .filter(folder => folder.id !== folderId)
        .map(folder => 
          folder.parentId === folderId ? { ...folder, parentId: parentId } : folder
        );
      
      setPasswords(updatedPasswords);
      updateFolderCounts(updatedPasswords, updatedFolders);
      
      if (selectedFolder === folderId) {
        setSelectedFolder('root');
      }
    } catch (err) {
      console.error('Error deleting folder:', err);
      setError('Failed to delete folder');
    }
  };

  const handleRenameFolder = async (folderId, newName) => {
    if (folderId === 'root' || !newName.trim()) return;
    
    try {
      const response = await folderAPI.update(folderId, { name: newName.trim() });
      const updatedFolder = response.data.data;
      
      const updatedFolders = folders.map(folder => 
        folder.id === folderId ? { ...folder, name: updatedFolder.name } : folder
      );
      
      setFolders(updatedFolders);
    } catch (err) {
      console.error('Error renaming folder:', err);
      setError('Failed to rename folder');
    }
  };

  // Entry management
  const handleAddEntry = async (entryData) => {
    try {
      const passwordData = {
        folder_id: entryData.folderId === 'root' ? null : entryData.folderId,
        website: entryData.website,
        username: entryData.username,
        password: entryData.password,
        notes: entryData.notes || null,
        otp_secret: entryData.otp_secret || null,
        attachments: entryData.attachments || []
      };
      
      const response = await passwordAPI.create(passwordData);
      const newEntry = response.data.data;
      
      const updatedPasswords = [...passwords, newEntry];
      setPasswords(updatedPasswords);
      updateFolderCounts(updatedPasswords, folders);
    } catch (err) {
      console.error('Error creating password:', err);
      setError('Failed to create password');
    }
  };

  const handleEditEntry = (id) => {
    const entry = passwords.find(p => p.id === id);
    setEditingEntry(entry);
    setEditModalOpen(true);
  };

  const handleSaveEntry = async (id, updatedData) => {
    try {
      const passwordData = {
        folder_id: updatedData.folder_id === 'root' ? null : updatedData.folder_id,
        website: updatedData.website,
        username: updatedData.username,
        password: updatedData.password,
        notes: updatedData.notes || null,
        otp_secret: updatedData.otp_secret || null,
        attachments: updatedData.attachments || []
      };
      
      await passwordAPI.update(id, passwordData);
      
      const updatedPasswords = passwords.map(password => 
        password.id === id 
          ? { ...password, ...updatedData }
          : password
      );
      setPasswords(updatedPasswords);
      updateFolderCounts(updatedPasswords, folders);
    } catch (err) {
      console.error('Error updating password:', err);
      setError('Failed to update password');
    }
  };

  const handleDeleteEntry = async (id) => {
    if (window.confirm('Are you sure you want to delete this entry?')) {
      try {
        await passwordAPI.delete(id);
        
        const updatedPasswords = passwords.filter(password => password.id !== id);
        setPasswords(updatedPasswords);
        updateFolderCounts(updatedPasswords, folders);
      } catch (err) {
        console.error('Error deleting password:', err);
        setError('Failed to delete password');
      }
    }
  };

  const handleMoveEntry = async (entryId, targetFolderId) => {
    try {
      const password = passwords.find(p => p.id === entryId);
      if (!password) return;
      
      const moveData = {
        folder_id: targetFolderId === 'root' ? null : targetFolderId
      };
      
      await passwordAPI.move(entryId, moveData);
      
      const updatedPasswords = passwords.map(password => 
        password.id === entryId 
          ? { ...password, folder_id: targetFolderId === 'root' ? null : targetFolderId }
          : password
      );
      setPasswords(updatedPasswords);
      updateFolderCounts(updatedPasswords, folders);
    } catch (err) {
      console.error('Error moving password:', err);
      setError('Failed to move password');
    }
  };

  // Handle search results
  const handleSearchResults = (results, term) => {
    setSearchResults(results);
    setSearchTerm(term);
    setIsSearchActive(term.length > 0);
  };

  // Filter passwords by selected folder, search, or show shared items
  const filteredPasswords = showSharedItems 
    ? sharedPasswords
    : isSearchActive
      ? searchResults
      : selectedFolder === 'root' 
        ? passwords 
        : passwords.filter(password => password.folder_id === selectedFolder);

  const currentFolder = folders.find(f => f.id === selectedFolder);
  const getListTitle = () => {
    if (showSharedItems) return 'Shared with me';
    if (isSearchActive) return `Search results for "${searchTerm}"`;
    return currentFolder?.name || 'All Items';
  };
  const listTitle = getListTitle();

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-lg font-semibold">Loading your vault...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-center">
          <div className="text-red-600 font-semibold mb-2">Error: {error}</div>
          <button onClick={loadData} className="cartoon-btn cartoon-btn-primary">Retry</button>
        </div>
      </div>
    );
  }

  return (
    <div className="w-full h-[calc(100vh-56px)] flex flex-col">
      <div className="flex flex-col md:flex-row flex-1 overflow-hidden bg-white relative">
        {/* Desktop Sidebar */}
        <aside className="hidden md:flex md:flex-col md:w-64 md:min-w-[200px] border-r border-black/20 bg-white p-4 overflow-y-auto">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-bold text-lg">Folders</h3>
            <button className="cartoon-btn cartoon-btn-primary p-1" title="Add Folder" onClick={() => handleAddFolder('New Folder')}>
              <Icons.Plus size={18} />
            </button>
          </div>
          <FolderTree 
            folders={folders}
            selectedFolder={selectedFolder}
            onSelectFolder={setSelectedFolder}
            onAddFolder={handleAddFolder}
            onDeleteFolder={handleDeleteFolder}
            onRenameFolder={handleRenameFolder}
            onMoveEntry={handleMoveEntry}
          />
        </aside>

        {/* Main content */}
        <main className="flex-1 flex flex-col items-center px-4 pt-4 pb-6 overflow-y-auto">
          {/* Mobile Folder Accordion */}
          <div className="md:hidden w-full mb-4">
            <div className="cartoon-border bg-white">
              <button 
                className="w-full flex items-center justify-between p-3" 
                onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              >
                <h3 className="font-bold text-base">Folders</h3>
                <Icons.ChevronDown size={20} className={`transform transition-transform ${mobileMenuOpen ? 'rotate-180' : ''}`} />
              </button>
              {mobileMenuOpen && (
                <div className="p-4 border-t border-black/20">
                  <FolderTree 
                    folders={folders}
                    selectedFolder={selectedFolder}
                    onSelectFolder={(folderId) => {
                      setSelectedFolder(folderId);
                      setMobileMenuOpen(false);
                    }}
                    onAddFolder={handleAddFolder}
                    onDeleteFolder={handleDeleteFolder}
                    onRenameFolder={handleRenameFolder}
                    onMoveEntry={handleMoveEntry}
                  />
                </div>
              )}
            </div>
          </div>

          {/* Search Bar */}
          <SearchBar 
            passwords={showSharedItems ? sharedPasswords : (selectedFolder === 'root' ? passwords : passwords.filter(p => p.folder_id === selectedFolder))}
            onFilteredResults={handleSearchResults}
            placeholder={showSharedItems ? "Search shared items..." : "Search passwords..."}
          />
          
          <div className="w-full max-w-2xl flex flex-col items-center text-center md:flex-row md:justify-between md:text-left mb-6">
            <div className="mb-4 md:mb-0">
              <h2 className="text-xl font-bold">{listTitle}</h2>
              <span className="text-black/50">({filteredPasswords.length})</span>
            </div>
            <div className="flex flex-col md:flex-row gap-2 items-center">
              <div className="flex flex-wrap gap-2">
                  <button className="cartoon-btn cartoon-btn-primary flex items-center gap-1" onClick={() => setAddModalOpen(true)}>
                    <Icons.Plus size={16} />
                    <span>Add Entry</span>
                  </button>
                  <button 
                    className={`cartoon-btn flex items-center gap-1 ${showSharedItems ? 'cartoon-btn-primary' : ''}`} 
                    onClick={() => setShowSharedItems(!showSharedItems)}
                  >
                    <Icons.Share size={16} />
                    <span>{showSharedItems ? 'My Items' : 'Shared Items'}</span>
                  </button>
                  {!showSharedItems && (
                    <>
                      <button 
                        className="cartoon-btn flex items-center gap-1" 
                        onClick={handleExportCSV}
                        title="Export passwords to CSV"
                      >
                        <Icons.Download size={16} />
                        <span>Export CSV</span>
                      </button>
                      <button 
                        className="cartoon-btn flex items-center gap-1" 
                        onClick={triggerFileInput}
                        title="Import passwords from CSV"
                      >
                        <Icons.Upload size={16} />
                        <span>Import CSV</span>
                      </button>
                      <input
                        id="csv-import-input"
                        type="file"
                        accept=".csv"
                        onChange={handleImportCSV}
                        style={{ display: 'none' }}
                      />
                    </>
                  )}
              </div>
            </div>
          </div>
          <div className="w-full max-w-2xl">
            {filteredPasswords.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16">
                <Icons.Lock size={48} className="mb-4" />
                <h4 className="text-lg font-semibold mb-2">{showSharedItems ? 'No shared items' : 'No entries in this folder'}</h4>
                <p className="mb-4 text-black/60">{showSharedItems ? 'No items have been shared with you yet.' : 'Get started by adding your first password entry.'}</p>
                {!showSharedItems && (
                  <button className="cartoon-btn cartoon-btn-primary flex items-center gap-1" onClick={() => setAddModalOpen(true)}>
                    <Icons.Plus size={16} />
                    <span>Add your first entry</span>
                  </button>
                )}
              </div>
            ) : (
              <div className="flex flex-col gap-3">
                {filteredPasswords.map(password => (
                  <PasswordItem
                    key={password.id}
                    id={password.id}
                    website={password.website}
                    username={password.username}
                    password={password.password}
                    notes={password.notes}
                    otp_secret={password.otp_secret}
                    attachments={password.attachments}
                    folderId={password.folderId}
                    onEdit={!showSharedItems ? handleEditEntry : null}
                    onDelete={!showSharedItems ? handleDeleteEntry : null}
                    onShare={!showSharedItems ? (id) => handleShareItem(id, 'password') : null}
                    isShared={showSharedItems}
                  />
                ))}
              </div>
            )}
          </div>
        </main>
      </div>
      {/* Modals */}
      <ShareModal
        isOpen={isShareModalOpen}
        onClose={() => {
          setShareModalOpen(false);
          setShareItem(null);
        }}
        itemId={shareItem?.id}
        itemType={shareItem?.type}
        onSuccess={handleShareSuccess}
      />
      <AddEntryModal
        isOpen={addModalOpen}
        onClose={() => setAddModalOpen(false)}
        onAdd={handleAddEntry}
        selectedFolder={selectedFolder}
        folders={folders}
        onAddFolder={handleAddFolder}
      />
      <EditEntryModal
        isOpen={editModalOpen}
        onClose={() => {
          setEditModalOpen(false);
          setEditingEntry(null);
        }}
        onSave={handleSaveEntry}
        entry={editingEntry}
        folders={folders}
        onAddFolder={handleAddFolder}
      />
    </div>
  );
}

export default Dashboard;