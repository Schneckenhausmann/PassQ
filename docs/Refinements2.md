# PassQ Refinements 2.0

## Overview
This document outlines the next phase of improvements for PassQ, focusing on enhanced security features, user experience improvements, and offline capabilities for both the web application and browser extensions.

## 1. Vault Auto-Lock Settings

### Frontend Settings Panel
- **Auto-lock timeout configuration**
  - Dropdown/slider for timeout duration: 5min, 15min, 30min, 1hr, 2hr, 4hr, 8hr, Never
  - Default: 15 minutes
  - Setting persisted in user preferences

- **Page refresh behavior**
  - Toggle: "Lock vault on page refresh"
  - Default: Enabled for security
  - When enabled, any page refresh/navigation requires re-authentication
  - When disabled, maintains session until timeout

### Implementation Details
- Add timeout management to frontend session handling
- Implement activity detection (mouse movement, keyboard input)
- Reset timer on user activity
- Show warning notification 1 minute before auto-lock
- Graceful session cleanup on timeout

## 2. Browser Extension Auto-Lock

### Extension Settings
- **Extension timeout configuration**
  - Independent from web app settings
  - Options: 1min, 5min, 15min, 30min, 1hr, 2hr, Never
  - Default: 5 minutes (more aggressive for security)

- **Lock triggers**
  - Inactivity timeout
  - Browser restart
  - Extension reload/update
  - System sleep/hibernate

### Implementation Details
- Add settings page to extension popup
- Implement background timer in service worker
- Clear sensitive data from memory on lock
- Require master password re-entry after lock
- Sync timeout settings across Chrome/Firefox if user logged in

## 2.1. Biometric Authentication

### Supported Biometric Methods
- **macOS Touch ID**: Fingerprint authentication on MacBooks
- **Windows Hello**: Fingerprint, facial recognition, and PIN on Windows devices
- **Android Fingerprint**: For mobile browser extensions (future consideration)
- **Hardware Security Keys**: FIDO2/WebAuthn compatible devices

### Biometric Settings
- **Enable biometric unlock**
  - Toggle in extension settings
  - Fallback to master password always available
  - Biometric data never leaves the device

- **Biometric timeout**
  - Independent timeout for biometric authentication
  - Options: 30sec, 1min, 5min, 15min, 30min
  - Default: 5 minutes
  - After timeout, requires full master password

### Implementation Strategy

#### WebAuthn Integration
```javascript
// Biometric authentication setup
const credential = await navigator.credentials.create({
  publicKey: {
    challenge: new Uint8Array(32),
    rp: { name: "PassQ Extension" },
    user: {
      id: userIdBuffer,
      name: userEmail,
      displayName: userName
    },
    pubKeyCredParams: [{ alg: -7, type: "public-key" }],
    authenticatorSelection: {
      authenticatorAttachment: "platform",
      userVerification: "required"
    }
  }
});
```

#### Platform-Specific Implementation
- **Chrome Extension**: Use WebAuthn API with platform authenticators
- **Firefox Extension**: Leverage WebAuthn support in Firefox
- **Fallback mechanism**: Always maintain master password option

### Security Considerations
- **Local verification only**: Biometric data never transmitted
- **Encrypted key storage**: Master key encrypted with biometric-derived key
- **Revocation support**: Easy disable/re-enable of biometric authentication
- **Multi-factor option**: Combine biometrics with additional factors

### User Experience
- **Setup wizard**: Guide users through biometric enrollment
- **Visual indicators**: Show biometric availability status
- **Quick unlock**: Single touch/scan to unlock extension
- **Graceful degradation**: Seamless fallback to password when biometrics unavailable

## 3. Offline Caching for Extensions

### Caching Strategy
- **Cache duration**: 24 hours maximum
- **Cache scope**: 
  - User's encrypted vault data
  - Recently accessed credentials (last 50)
  - User preferences and settings

### Cache Management
- **Storage location**: Browser's IndexedDB for larger storage capacity
- **Encryption**: All cached data encrypted with user's master key
- **Cache invalidation**:
  - Automatic after 24 hours
  - Manual refresh option in settings
  - Clear cache on logout

### Offline Functionality
- **Read-only access** to cached credentials
- **Visual indicators** showing offline mode
- **Sync queue** for changes made offline
- **Conflict resolution** when reconnecting

### Implementation Details
```javascript
// Cache structure
{
  timestamp: Date.now(),
  encryptedVault: "...",
  lastSync: "2024-01-15T10:30:00Z",
  pendingChanges: [],
  userSettings: {...}
}
```

## 4. Extension-to-Database Synchronization

### Bidirectional Sync
- **Real-time updates** when online
- **Conflict detection** and resolution
- **Change tracking** with timestamps
- **Optimistic updates** with rollback capability

### Sync Operations
- **Create**: New credentials added via extension
- **Update**: Modified credentials (password changes, notes, etc.)
- **Delete**: Removed credentials
- **Metadata**: Last accessed, usage count, etc.

### API Endpoints
```
POST /api/sync/credentials
PUT /api/sync/credentials/:id
DELETE /api/sync/credentials/:id
GET /api/sync/changes?since=timestamp
```

### Conflict Resolution
- **Last-write-wins** for simple conflicts
- **User prompt** for complex conflicts
- **Merge strategies** for non-conflicting changes
- **Backup creation** before applying remote changes

## 5. Popup Window Feature

### Detached Window Mode
- **"Pop out" button** in extension popup header
- **Window specifications**:
  - Width: 400px
  - Height: 600px
  - Resizable: Yes
  - Always on top: Optional setting

### Window Management
- **Single instance**: Only one detached window at a time
- **State persistence**: Remember window position/size
- **Focus management**: Bring to front when activated
- **Auto-close**: Close with main browser or on timeout

### Implementation Details
```javascript
// Chrome Extension API
chrome.windows.create({
  url: 'popup.html?detached=true',
  type: 'popup',
  width: 400,
  height: 600,
  focused: true
});
```

## 6. Technical Implementation Plan

### Phase 1: Settings Infrastructure
1. Add settings storage schema
2. Create settings UI components
3. Implement timeout management
4. Add auto-lock functionality
5. Integrate biometric authentication setup

### Phase 2: Offline Capabilities
1. Implement IndexedDB caching layer
2. Add offline detection
3. Create sync queue mechanism
4. Build conflict resolution system

### Phase 3: Extension Enhancements
1. Add detached window functionality
2. Implement extension-specific settings
3. Create bidirectional sync system
4. Add offline mode indicators
5. Implement biometric authentication (WebAuthn)
6. Add platform-specific biometric support

### Phase 4: Testing & Polish
1. Comprehensive testing across browsers
2. Performance optimization
3. Security audit
4. User experience refinements

## 7. Security Considerations

### Data Protection
- All cached data encrypted with user's master key
- Secure key derivation for offline storage
- Memory cleanup on lock/timeout
- Protection against timing attacks
- Biometric authentication keys stored in secure hardware
- No biometric data transmission or cloud storage

### Session Management
- Secure token handling
- Proper session invalidation
- Protection against session fixation
- CSRF protection for sync operations

### Offline Security
- Limited offline session duration
- Encrypted local storage only
- No sensitive operations in offline mode
- Secure sync verification

## 8. User Experience Improvements

### Visual Indicators
- Lock status in UI
- Offline mode indicators
- Sync status notifications
- Timeout warnings

### Accessibility
- Keyboard navigation for all features
- Screen reader compatibility
- High contrast mode support
- Proper ARIA labels

### Performance
- Lazy loading for large vaults
- Efficient caching strategies
- Minimal memory footprint
- Fast startup times

## 9. Migration Strategy

### Existing Users
- Automatic migration to new settings schema
- Default values for new settings
- Backward compatibility during transition
- Clear communication about new features

### Data Migration
- Gradual rollout of caching features
- Optional offline mode initially
- User consent for local data storage
- Easy opt-out mechanisms

## 10. Success Metrics

### Security Metrics
- Reduced session hijacking incidents
- Improved compliance with security policies
- Faster incident response times

### User Experience Metrics
- Reduced authentication friction
- Increased extension usage
- Improved user satisfaction scores
- Reduced support tickets

### Performance Metrics
- Faster credential access times
- Reduced server load
- Improved offline functionality usage
- Better sync reliability

---

*This document serves as a comprehensive roadmap for implementing the next generation of PassQ features, prioritizing security, usability, and reliability.*