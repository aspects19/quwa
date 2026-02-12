import { useState, useCallback } from 'react';
import { Upload, X, FileText, Image, CheckCircle, AlertCircle, Loader2 } from 'lucide-react';
import { getValidJWT } from '@/lib/appwrite';

const BACKEND_URL = import.meta.env.VITE_BACKEND_URL || 'http://localhost:3000';

interface UploadedFile {
  id: string;
  name: string;
  status: 'uploading' | 'processing' | 'completed' | 'failed';
  progress: number;
  error?: string;
}

const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50MB
const ALLOWED_TYPES = ['application/pdf', 'image/jpeg', 'image/jpg', 'image/png'];
const ALLOWED_EXTENSIONS = ['pdf', 'jpg', 'jpeg', 'png'];

export default function FileUpload() {
  const [files, setFiles] = useState<UploadedFile[]>([]);
  const [isDragging, setIsDragging] = useState(false);

  const validateFile = (file: File): string | null => {
    // Check file size
    if (file.size > MAX_FILE_SIZE) {
      return 'File size exceeds 50MB limit';
    }

    // Check file type
    const extension = file.name.split('.').pop()?.toLowerCase();
    if (!extension || !ALLOWED_EXTENSIONS.includes(extension)) {
      return 'Only PDF and image files (JPG, PNG) are allowed';
    }

    if (!ALLOWED_TYPES.includes(file.type) && !file.type.startsWith('image/')) {
      return 'Invalid file type';
    }

    return null;
  };

  const uploadFile = async (file: File) => {
    const fileId = `${Date.now()}-${file.name}`;
    
    // Add file to state
    setFiles(prev => [...prev, {
      id: fileId,
      name: file.name,
      status: 'uploading',
      progress: 0,
    }]);

    try {
      const jwtToken = await getValidJWT();
      const formData = new FormData();
      formData.append('file', file);

      const response = await fetch(`${BACKEND_URL}/api/upload`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${jwtToken}`,
        },
        body: formData,
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || 'Upload failed');
      }

      await response.json();
      
      // Update to processing status
      setFiles(prev => prev.map(f => 
        f.id === fileId 
          ? { ...f, status: 'processing', progress: 100 }
          : f
      ));

      // Poll for completion (backend processes asynchronously)
      // For now, just mark as completed after a delay
      setTimeout(() => {
        setFiles(prev => prev.map(f => 
          f.id === fileId 
            ? { ...f, status: 'completed' }
            : f
        ));
      }, 2000);

    } catch (error) {
      console.error('Upload error:', error);
      setFiles(prev => prev.map(f => 
        f.id === fileId 
          ? { 
              ...f, 
              status: 'failed', 
              error: error instanceof Error ? error.message : 'Upload failed'
            }
          : f
      ));
    }
  };

  const handleFiles = useCallback((fileList: FileList | null) => {
    if (!fileList) return;

    Array.from(fileList).forEach(file => {
      const error = validateFile(file);
      if (error) {
        // Add as failed file
        setFiles(prev => [...prev, {
          id: `${Date.now()}-${file.name}`,
          name: file.name,
          status: 'failed',
          progress: 0,
          error,
        }]);
      } else {
        uploadFile(file);
      }
    });
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    handleFiles(e.dataTransfer.files);
  }, [handleFiles]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const removeFile = (id: string) => {
    setFiles(prev => prev.filter(f => f.id !== id));
  };

  const getFileIcon = (fileName: string) => {
    const ext = fileName.split('.').pop()?.toLowerCase();
    if (ext === 'pdf') return FileText;
    return Image;
  };

  const getStatusIcon = (status: UploadedFile['status']) => {
    switch (status) {
      case 'uploading':
      case 'processing':
        return Loader2;
      case 'completed':
        return CheckCircle;
      case 'failed':
        return AlertCircle;
    }
  };

  const getStatusColor = (status: UploadedFile['status']) => {
    switch (status) {
      case 'uploading':
      case 'processing':
        return 'text-blue-400';
      case 'completed':
        return 'text-green-400';
      case 'failed':
        return 'text-red-400';
    }
  };

  return (
    <div className="w-full max-w-2xl mx-auto space-y-4">
      {/* Drop Zone */}
      <div
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        className={`
          relative border-2 border-dashed rounded-2xl p-8 transition-all
          ${isDragging 
            ? 'border-primary bg-primary/10 scale-105' 
            : 'border-white/20 bg-white/5'
          }
          hover:border-primary/50 hover:bg-white/10 cursor-pointer
        `}
      >
        <input
          type="file"
          multiple
          accept=".pdf,.jpg,.jpeg,.png"
          onChange={(e) => handleFiles(e.target.files)}
          className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
        />
        
        <div className="flex flex-col items-center gap-3 pointer-events-none">
          <div className="p-4 rounded-full bg-primary/20">
            <Upload className="w-8 h-8 text-primary" />
          </div>
          <div className="text-center">
            <p className="text-white font-medium mb-1">
              Drop medical files here or click to browse
            </p>
            <p className="text-sm text-white/60">
              PDF, JPG, PNG up to 50MB
            </p>
          </div>
        </div>
      </div>

      {/* File List */}
      {files.length > 0 && (
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-white/80">Uploaded Files</h3>
          <div className="space-y-2">
            {files.map((file) => {
              const FileIcon = getFileIcon(file.name);
              const StatusIcon = getStatusIcon(file.status);
              const statusColor = getStatusColor(file.status);

              return (
                <div
                  key={file.id}
                  className="flex items-center gap-3 p-3 rounded-xl bg-white/5 border border-white/10 backdrop-blur-md"
                >
                  <FileIcon className="w-5 h-5 text-white/60 shrink-0" />
                  
                  <div className="flex-1 min-w-0">
                    <p className="text-sm text-white truncate">{file.name}</p>
                    {file.error ? (
                      <p className="text-xs text-red-400 mt-1">{file.error}</p>
                    ) : (
                      <p className="text-xs text-white/60 mt-1 capitalize">
                        {file.status}
                      </p>
                    )}
                  </div>

                  <StatusIcon 
                    className={`w-5 h-5 shrink-0 ${statusColor} ${
                      (file.status === 'uploading' || file.status === 'processing') 
                        ? 'animate-spin' 
                        : ''
                    }`}
                  />

                  <button
                    onClick={() => removeFile(file.id)}
                    className="p-1 rounded-lg hover:bg-white/10 transition-colors"
                    aria-label="Remove file"
                  >
                    <X className="w-4 h-4 text-white/60" />
                  </button>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
