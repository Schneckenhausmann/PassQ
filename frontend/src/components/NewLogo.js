import React from 'react';

function NewLogo({ size = 40, className = '' }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 100 100"
      className={className}
      xmlns="http://www.w3.org/2000/svg"
      aria-label="New PassQ Logo"
    >
      {/* Outer circle with thick black border */}
      <circle
        cx="50"
        cy="50"
        r="45"
        fill="#fff"
        stroke="#000"
        strokeWidth="6"
      />
      {/* Stylized P and Q letters combined */}
      <text
        x="50%"
        y="55%"
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize="48"
        fontWeight="900"
        fontFamily="'Inter', 'Arial', sans-serif"
        fill="#000"
        stroke="#fff"
        strokeWidth="2"
        style={{ paintOrder: 'stroke fill' }}
      >
        PQ
      </text>
      {/* Tail for Q */}
      <path
        d="M62 62 Q70 70 80 80"
        stroke="#000"
        strokeWidth="6"
        fill="none"
        strokeLinecap="round"
      />
      {/* Shadow ellipse */}
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

export default NewLogo;