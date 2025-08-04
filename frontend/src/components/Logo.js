import React from 'react';

function Logo({ size = 40, className = '' }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 100 100"
      className={`logo-wiggle ${className}`}
      xmlns="http://www.w3.org/2000/svg"
      aria-label="PassQ Logo"
    >
      {/* Cartoon bold circle */}
      <circle
        cx="50"
        cy="50"
        r="45"
        fill="#fff"
        stroke="#000"
        strokeWidth="6"
      />
      {/* Quirky bold Q */}
      <text
        x="50%"
        y="56%"
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize="56"
        fontWeight="900"
        fontFamily="'Comic Sans MS', 'Comic Neue', 'Inter', 'Arial', sans-serif"
        fill="#000"
        stroke="#fff"
        strokeWidth="2"
        style={{ paintOrder: 'stroke fill' }}
      >
        Q
      </text>
      {/* Quirky tail for Q */}
      <path
        d="M62 62 Q70 70 80 80"
        stroke="#000"
        strokeWidth="6"
        fill="none"
        strokeLinecap="round"
      />
      {/* Cartoon shadow */}
      <ellipse
        cx="60"
        cy="90"
        rx="18"
        ry="6"
        fill="#000"
        opacity="0.13"
      />
    </svg>
  );
}

export default Logo;