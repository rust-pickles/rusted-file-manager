import React from 'react';

interface BrandIconProps {
  className?: string;
  size?: number;
}

export const BrandIcon: React.FC<BrandIconProps> = ({ className = 'w-4 h-4', size = 16 }) => {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      aria-label="RUSTED Icon"
    >
      {/* Precision geometric isometric faceted prism representing Rusted core */}
      <path
        d="M12 2L20.5 6.9V17.1L12 22L3.5 17.1V6.9L12 2Z"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinejoin="round"
        className="opacity-90"
      />
      <path
        d="M12 22V12M12 12L20.5 6.9M12 12L3.5 6.9"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
        className="opacity-60"
      />
      <path
        d="M12 7.5L16.25 10V14.5L12 17L7.75 14.5V10L12 7.5Z"
        fill="currentColor"
        fillOpacity="0.2"
        stroke="currentColor"
        strokeWidth="1.25"
        strokeLinejoin="round"
      />
    </svg>
  );
};
